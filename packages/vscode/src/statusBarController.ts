import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { TerminalLauncher } from './terminalLauncher';
import { ScannerBridge } from './scannerBridge';
import { DetectedAgent } from './types';

export class StatusBarController {
  private statusBarItem: vscode.StatusBarItem;
  private disposables: vscode.Disposable[] = [];
  private nextRefreshTime: number | undefined;

  constructor(
    private context: vscode.ExtensionContext,
    private agentService: AgentService,
    private terminalLauncher: TerminalLauncher,
    private scannerBridge: ScannerBridge
  ) {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );
    this.statusBarItem.command = 'odScanner.statusBarClicked';
    this.context.subscriptions.push(this.statusBarItem);

    const clickCmd = vscode.commands.registerCommand(
      'odScanner.statusBarClicked',
      () => this.showMenu()
    );
    this.context.subscriptions.push(clickCmd);

    this.disposables.push(
      { dispose: this.agentService.onChange(() => this.refresh()) }
    );
  }

  setNextRefreshTime(timestamp: number | undefined): void {
    this.nextRefreshTime = timestamp;
  }

  refresh(): void {
    const available = this.agentService.getAvailable();
    const count = available.length;

    let tooltip = '';
    if (this.nextRefreshTime) {
      const remaining = Math.max(0, Math.ceil((this.nextRefreshTime - Date.now()) / 1000));
      tooltip = `Next refresh in ${remaining}s\n`;
    }

    if (count === 0) {
      this.statusBarItem.text = '$(robot) Install \u2192';
      this.statusBarItem.tooltip = tooltip + 'No AI agents detected. Click to install.';
      this.statusBarItem.command = 'odScanner.statusBarClicked';
    } else {
      this.statusBarItem.text = `$(robot) ${count}`;
      this.statusBarItem.tooltip = tooltip + `${count} AI agent${count > 1 ? 's' : ''} available. Click to launch.`;
      this.statusBarItem.command = 'odScanner.statusBarClicked';
    }
    this.statusBarItem.show();
  }

  setScanning(): void {
    this.statusBarItem.text = '$(robot) ...';
    this.statusBarItem.tooltip = 'Scanning for AI agents...';
    this.statusBarItem.show();
  }

  setError(message: string): void {
    this.statusBarItem.text = '$(robot) !';
    this.statusBarItem.tooltip = `Scanner error: ${message}`;
    this.statusBarItem.show();
  }

  private async showMenu(): Promise<void> {
    const available = this.agentService.getAvailable();
    const config = vscode.workspace.getConfiguration('odScanner');
    const showUnavailable = config.get<boolean>('showUnavailable', false);

    if (available.length === 0) {
      const choice = await vscode.window.showQuickPick(
        [
          { label: '$(refresh) Refresh Agents', action: 'refresh' as const },
          { label: '$(gear) Open Settings', action: 'settings' as const },
          { label: '$(book) View README', action: 'readme' as const },
        ],
        { placeHolder: 'No AI agents detected. What would you like to do?' }
      );
      if (!choice) { return; }
      if (choice.action === 'refresh') {
        await vscode.commands.executeCommand('odScanner.refreshAgents');
      } else if (choice.action === 'settings') {
        await vscode.commands.executeCommand('odScanner.openSettings');
      } else if (choice.action === 'readme') {
        await vscode.env.openExternal(
          vscode.Uri.parse('https://github.com/gushuaialan1/od-cli-scanner#readme')
        );
      }
      return;
    }

    const sortedAgents = this.agentService.getRecentAgents();
    const items: Array<{ label: string; description: string; detail: string; agent?: DetectedAgent; action?: string }> = sortedAgents.map((agent) => ({
      label: `$(play) ${agent.name}`,
      description: agent.version || '',
      detail: agent.path || '',
      agent,
    }));

    if (showUnavailable) {
      const unavailable = this.agentService
        .getAll()
        .filter((a) => !a.available);
      if (unavailable.length > 0) {
        items.push({
          label: '',
          description: '',
          detail: '---',
          agent: undefined as any,
        });
        for (const agent of unavailable) {
          items.push({
            label: `$(debug-disconnect) ${agent.name}`,
            description: 'Not installed',
            detail: agent.install_url || '',
            agent: undefined as any,
          });
        }
      }
    }

    items.push({
      label: '',
      description: '',
      detail: '---',
      agent: undefined as any,
    });
    items.push({
      label: '$(refresh) Refresh Agents',
      description: '',
      detail: '',
      agent: undefined as any,
      action: 'refresh' as const,
    });
    items.push({
      label: '$(gear) Settings...',
      description: '',
      detail: '',
      agent: undefined as any,
      action: 'settings' as const,
    });

    const picked = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select an AI agent to launch',
    });

    if (!picked) { return; }

    if ((picked as any).action === 'refresh') {
      await vscode.commands.executeCommand('odScanner.refreshAgents');
      return;
    }
    if ((picked as any).action === 'settings') {
      await vscode.commands.executeCommand('odScanner.openSettings');
      return;
    }

    if (picked.agent) {
      const selectedModel = await this.pickModelIfNeeded(picked.agent);
      this.terminalLauncher.spawn(picked.agent, undefined, selectedModel);
      this.agentService.recordUsage(picked.agent.id);
    }
  }

  private async pickModelIfNeeded(agent: DetectedAgent): Promise<string | undefined> {
    const models = this.agentService.getModels(agent.id);
    if (!models || models.length <= 1) {
      return undefined;
    }
    const modelItems = models.map((m) => ({
      label: m.label || m.id,
      description: m.id,
      modelId: m.id,
    }));
    const picked = await vscode.window.showQuickPick(modelItems, {
      placeHolder: `Select a model for ${agent.name}`,
    });
    return picked?.modelId;
  }

  dispose(): void {
    this.statusBarItem.dispose();
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
