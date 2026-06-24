import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { ScannerBridge } from './scannerBridge';
import { StatusBarController } from './statusBarController';
import { CommandController } from './commandController';
import { ContextMenuController } from './contextMenuController';
import { AgentTreeProvider } from './agentTreeProvider';
import { TerminalLauncher } from './terminalLauncher';
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

  // Register launch by id command (used by tree view)
  const launchByIdCmd = vscode.commands.registerCommand(
    'odScanner.launchAgentById',
    (agentId: string) => {
      const agent = agentService.getById(agentId);
      if (agent) {
        terminalLauncher.spawn(agent);
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

  // Auto-refresh
  const config = vscode.workspace.getConfiguration('odScanner');
  if (config.get<boolean>('autoRefresh', true)) {
    const intervalSec = config.get<number>('refreshInterval', 60);
    refreshTimer = setInterval(() => performScan(), intervalSec * 1000);
  }

  // Re-register when configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('odScanner')) {
        restartAutoRefresh(context);
      }
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
  } catch (err) {
    const message = err instanceof ScannerError ? err.message : String(err);
    statusBarController?.setError(message);
    outputChannel.appendLine(`[Scan Error] ${message}`);
  }
}

function restartAutoRefresh(context: vscode.ExtensionContext): void {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = undefined;
  }
  const config = vscode.workspace.getConfiguration('odScanner');
  if (config.get<boolean>('autoRefresh', true)) {
    const intervalSec = config.get<number>('refreshInterval', 60);
    refreshTimer = setInterval(() => performScan(), intervalSec * 1000);
  }
}
