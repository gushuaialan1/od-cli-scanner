import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { ScannerBridge } from './scannerBridge';
import { StatusBarController } from './statusBarController';
import { CommandController } from './commandController';
import { ContextMenuController } from './contextMenuController';
import { AgentTreeProvider } from './agentTreeProvider';
import { TerminalLauncher } from './terminalLauncher';
import { ChatTerminalPanel } from './chatTerminalPanel';
import { ScannerError } from './types';

let agentService: AgentService;
let scannerBridge: ScannerBridge;
let statusBarController: StatusBarController;
let commandController: CommandController;
let contextMenuController: ContextMenuController;
let agentTreeProvider: AgentTreeProvider;
let terminalLauncher: TerminalLauncher;
let outputChannel: vscode.OutputChannel;
let refreshTimer: NodeJS.Timeout | undefined;
let isPaused = false;

export function activate(context: vscode.ExtensionContext): void {
  outputChannel = vscode.window.createOutputChannel('OD Scanner');
  context.subscriptions.push(outputChannel);

  agentService = new AgentService();
  agentService.bindContext(context);
  terminalLauncher = new TerminalLauncher();
  scannerBridge = new ScannerBridge(outputChannel);
  statusBarController = new StatusBarController(
    context,
    agentService,
    terminalLauncher,
    scannerBridge
  );
  commandController = new CommandController(
    context,
    agentService,
    terminalLauncher,
    scannerBridge
  );
  contextMenuController = new ContextMenuController(
    context,
    agentService,
    terminalLauncher,
    scannerBridge
  );

  // Tree view for agent details
  agentTreeProvider = new AgentTreeProvider(agentService, terminalLauncher);
  const treeView = vscode.window.createTreeView('odScannerAgents', {
    treeDataProvider: agentTreeProvider,
  });
  context.subscriptions.push(treeView);

  // Chat terminal panel
  const chatTerminalPanel = new ChatTerminalPanel(context, agentService);
  const chatView = vscode.window.registerWebviewViewProvider(
    'odScanner.chatTerminal',
    chatTerminalPanel
  );
  context.subscriptions.push(chatView);

  // Register launch by id command (used by tree view)
  const launchByIdCmd = vscode.commands.registerCommand(
    'odScanner.launchAgentById',
    async (agentId: string) => {
      const agent = agentService.getById(agentId);
      if (agent) {
        const selectedModel = await pickModelIfNeeded(agent);
        terminalLauncher.spawn(agent, undefined, selectedModel);
        agentService.recordUsage(agentId);
      }
    }
  );
  context.subscriptions.push(launchByIdCmd);

  // Refresh tree view command
  const refreshTreeCmd = vscode.commands.registerCommand(
    'odScanner.refreshTreeView',
    () => agentTreeProvider.refresh()
  );
  context.subscriptions.push(refreshTreeCmd);

  // Initial scan
  performScan();
  vscode.commands.executeCommand('setContext', 'odScanner.loaded', true);

  // Auto-refresh
  startAutoRefresh();

  // Re-register when configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('odScanner')) {
        restartAutoRefresh();
      }
    })
  );

  // Listen for workspace folder changes to refresh tree
  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      agentTreeProvider.refresh();
    })
  );
}

export function deactivate(): void {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = undefined;
  }
  statusBarController?.dispose();
  outputChannel?.dispose();
}

async function performScan(): Promise<void> {
  statusBarController?.setScanning();
  try {
    const agents = await scannerBridge.scan();
    agentService.update(agents);
    statusBarController?.refresh();
    agentTreeProvider?.refresh();
  } catch (err) {
    const message = err instanceof ScannerError ? err.message : String(err);
    statusBarController?.setError(message);
    outputChannel.appendLine(`[Scan Error] ${message}`);
  }
}

function startAutoRefresh(): void {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = undefined;
  }
  const config = vscode.workspace.getConfiguration('odScanner');
  if (isPaused || !config.get<boolean>('autoRefresh', true)) {
    statusBarController?.setNextRefreshTime(undefined);
    return;
  }
  const intervalSec = config.get<number>('refreshInterval', 60);
  const intervalMs = intervalSec * 1000;
  const nextTime = Date.now() + intervalMs;
  statusBarController?.setNextRefreshTime(nextTime);
  refreshTimer = setInterval(() => {
    if (isPaused) { return; }
    performScan();
    const next = Date.now() + intervalMs;
    statusBarController?.setNextRefreshTime(next);
  }, intervalMs);
}

function restartAutoRefresh(): void {
  startAutoRefresh();
}

async function pickModelIfNeeded(agent: import('./types').DetectedAgent): Promise<string | undefined> {
  const models = agentService.getModels(agent.id);
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
