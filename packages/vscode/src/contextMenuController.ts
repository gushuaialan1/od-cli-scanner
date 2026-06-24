import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { TerminalLauncher } from './terminalLauncher';
import { ScannerBridge } from './scannerBridge';
import { DetectedAgent, ScannerError } from './types';

export class ContextMenuController {
  private disposables: vscode.Disposable[] = [];

  constructor(
    private context: vscode.ExtensionContext,
    private agentService: AgentService,
    private terminalLauncher: TerminalLauncher,
    private scannerBridge: ScannerBridge
  ) {
    this.registerCommands();
  }

  private registerCommands(): void {
    const openWithCmd = vscode.commands.registerCommand(
      'odScanner.openWithAgent',
      (uri: vscode.Uri, selectedUris: vscode.Uri[]) => this.openWithAgent(uri, selectedUris)
    );
    this.context.subscriptions.push(openWithCmd);
  }

  private async openWithAgent(uri?: vscode.Uri, selectedUris?: vscode.Uri[]): Promise<void> {
    const available = this.agentService.getAvailable();
    if (available.length === 0) {
      const action = await vscode.window.showWarningMessage(
        'No AI agents detected.',
        'Refresh',
        'Open Settings'
      );
      if (action === 'Refresh') {
        await vscode.commands.executeCommand('odScanner.refreshAgents');
      } else if (action === 'Open Settings') {
        await vscode.commands.executeCommand('odScanner.openSettings');
      }
      return;
    }

    const uris = selectedUris && selectedUris.length > 0 ? selectedUris : uri ? [uri] : [];
    const contextPaths = uris.map((u) => u.fsPath);

    const items = available.map((agent) => ({
      label: agent.name,
      description: agent.version || '',
      detail: agent.path || '',
      agent,
    }));

    const picked = await vscode.window.showQuickPick(items, {
      placeHolder: contextPaths.length > 0
        ? `Select an AI agent to open ${contextPaths.length} item(s)`
        : 'Select an AI agent to launch',
    });

    if (!picked) { return; }

    const prompt = await vscode.window.showInputBox({
      placeHolder: 'Optional prompt (e.g., "Review this file")',
      prompt: 'Enter a prompt to pass to the agent, or leave blank',
    });

    this.launchAgent(picked.agent, contextPaths, prompt || undefined);
  }

  private launchAgent(agent: DetectedAgent, contextPaths: string[], prompt?: string): void {
    const config = vscode.workspace.getConfiguration('odScanner');
    const globalArgs = config.get<string[]>('launchArgs', []);
    const args: string[] = [...globalArgs];

    if (contextPaths.length > 0) {
      args.push(...contextPaths);
    }
    if (prompt) {
      args.push(prompt);
    }

    this.terminalLauncher.spawnWithArgs(agent, args);
    this.agentService.recordUsage(agent.id);
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
