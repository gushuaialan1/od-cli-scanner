import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { TerminalLauncher } from './terminalLauncher';
import { ScannerBridge } from './scannerBridge';
import { DetectedAgent } from './types';

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

    // Determine workspace folder for the selected file(s)
    let workspaceFolder: vscode.WorkspaceFolder | undefined;
    if (uris.length > 0) {
      workspaceFolder = vscode.workspace.getWorkspaceFolder(uris[0]);
    }

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

    const selectedModel = await this.pickModelIfNeeded(picked.agent);

    const prompt = await vscode.window.showInputBox({
      placeHolder: 'Optional prompt (e.g., "Review this file")',
      prompt: 'Enter a prompt to pass to the agent, or leave blank',
    });

    this.launchAgent(picked.agent, contextPaths, prompt || undefined, workspaceFolder, selectedModel);
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

  private launchAgent(
    agent: DetectedAgent,
    contextPaths: string[],
    prompt?: string,
    workspaceFolder?: vscode.WorkspaceFolder,
    model?: string
  ): void {
    const config = vscode.workspace.getConfiguration('odScanner', workspaceFolder?.uri);
    const globalArgs = config.get<string[]>('launchArgs', []);
    const args: string[] = [...globalArgs];

    if (contextPaths.length > 0) {
      args.push(...contextPaths);
    }
    if (prompt) {
      args.push(prompt);
    }

    this.terminalLauncher.spawnWithArgs(agent, args, model, workspaceFolder);
    this.agentService.recordUsage(agent.id);
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
