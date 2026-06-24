import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { TerminalLauncher } from './terminalLauncher';
import { ScannerBridge } from './scannerBridge';
import { DetectedAgent } from './types';

export class CommandController {
  constructor(
    private context: vscode.ExtensionContext,
    private agentService: AgentService,
    private terminalLauncher: TerminalLauncher,
    private scannerBridge: ScannerBridge
  ) {
    this.registerCommands();
  }

  private registerCommands(): void {
    const launchCmd = vscode.commands.registerCommand(
      'odScanner.launchAgent',
      () => this.launchAgent()
    );
    const refreshCmd = vscode.commands.registerCommand(
      'odScanner.refreshAgents',
      () => this.refreshAgents()
    );
    const settingsCmd = vscode.commands.registerCommand(
      'odScanner.openSettings',
      () => this.openSettings()
    );
    const pauseResumeCmd = vscode.commands.registerCommand(
      'odScanner.pauseResumeAutoRefresh',
      () => this.pauseResumeAutoRefresh()
    );

    this.context.subscriptions.push(launchCmd, refreshCmd, settingsCmd, pauseResumeCmd);
  }

  private async launchAgent(): Promise<void> {
    const available = this.agentService.getAvailable();
    if (available.length === 0) {
      const action = await vscode.window.showWarningMessage(
        'No AI agents detected.',
        'Refresh',
        'Open Settings'
      );
      if (action === 'Refresh') {
        await this.refreshAgents();
      } else if (action === 'Open Settings') {
        await this.openSettings();
      }
      return;
    }

    const config = vscode.workspace.getConfiguration('odScanner');
    const defaultAgentId = config.get<string>('defaultAgent', '');

    const items = this.agentService.getRecentAgents().map((agent) => ({
      label: agent.name,
      description: agent.version || '',
      detail: agent.path || '',
      agent,
    }));

    const picked = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select an AI agent to launch',
      ...(defaultAgentId ? { activeItems: items.filter((i) => i.agent.id === defaultAgentId) } : {}),
    });

    if (!picked) { return; }

    // Model selection if agent supports multiple models
    const selectedModel = await this.pickModelIfNeeded(picked.agent);

    const prompt = await vscode.window.showInputBox({
      placeHolder: 'Optional prompt (e.g., "Refactor this module")',
      prompt: 'Enter a prompt to pass to the agent, or leave blank',
    });

    const globalArgs = config.get<string[]>('launchArgs', []);
    const args = [...globalArgs];
    if (prompt) {
      args.push(prompt);
    }

    if (args.length > 0) {
      this.terminalLauncher.spawnWithArgs(picked.agent, args, selectedModel);
    } else {
      this.terminalLauncher.spawn(picked.agent, undefined, selectedModel);
    }
    this.agentService.recordUsage(picked.agent.id);
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

  private async refreshAgents(): Promise<void> {
    vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Window,
        title: 'Scanning AI agents...',
        cancellable: false,
      },
      async () => {
        try {
          const agents = await this.scannerBridge.scan();
          this.agentService.update(agents);
          const available = this.agentService.getAvailable();
          vscode.window.showInformationMessage(
            `Found ${available.length} AI agent${available.length !== 1 ? 's' : ''}.`
          );
        } catch (err) {
          vscode.window.showErrorMessage(
            `Agent scan failed: ${(err as Error).message}`
          );
        }
      }
    );
  }

  private async openSettings(): Promise<void> {
    await vscode.commands.executeCommand(
      'workbench.action.openSettings',
      'odScanner'
    );
  }

  private async pauseResumeAutoRefresh(): Promise<void> {
    const config = vscode.workspace.getConfiguration('odScanner');
    const current = config.get<boolean>('autoRefresh', true);
    await config.update('autoRefresh', !current, true);
    const state = !current ? 'enabled' : 'disabled';
    vscode.window.showInformationMessage(`Auto refresh ${state}.`);
  }
}
