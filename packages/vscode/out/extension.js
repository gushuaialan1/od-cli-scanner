"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const agentService_1 = require("./agentService");
const scannerBridge_1 = require("./scannerBridge");
const statusBarController_1 = require("./statusBarController");
const commandController_1 = require("./commandController");
const contextMenuController_1 = require("./contextMenuController");
const agentTreeProvider_1 = require("./agentTreeProvider");
const terminalLauncher_1 = require("./terminalLauncher");
const types_1 = require("./types");
let agentService;
let scannerBridge;
let statusBarController;
let commandController;
let contextMenuController;
let agentTreeProvider;
let terminalLauncher;
let outputChannel;
let refreshTimer;
function activate(context) {
    outputChannel = vscode.window.createOutputChannel('OD Scanner');
    context.subscriptions.push(outputChannel);
    agentService = new agentService_1.AgentService();
    agentService.bindContext(context);
    terminalLauncher = new terminalLauncher_1.TerminalLauncher();
    scannerBridge = new scannerBridge_1.ScannerBridge(outputChannel);
    statusBarController = new statusBarController_1.StatusBarController(context, agentService, terminalLauncher, scannerBridge);
    commandController = new commandController_1.CommandController(context, agentService, terminalLauncher, scannerBridge);
    contextMenuController = new contextMenuController_1.ContextMenuController(context, agentService, terminalLauncher, scannerBridge);
    // Tree view for agent details
    agentTreeProvider = new agentTreeProvider_1.AgentTreeProvider(agentService, terminalLauncher);
    const treeView = vscode.window.createTreeView('odScannerAgents', {
        treeDataProvider: agentTreeProvider,
    });
    context.subscriptions.push(treeView);
    // Register launch by id command (used by tree view)
    const launchByIdCmd = vscode.commands.registerCommand('odScanner.launchAgentById', (agentId) => {
        const agent = agentService.getById(agentId);
        if (agent) {
            terminalLauncher.spawn(agent);
            agentService.recordUsage(agentId);
        }
    });
    context.subscriptions.push(launchByIdCmd);
    // Refresh tree view command
    const refreshTreeCmd = vscode.commands.registerCommand('odScanner.refreshTreeView', () => agentTreeProvider.refresh());
    context.subscriptions.push(refreshTreeCmd);
    // Initial scan
    performScan();
    // Auto-refresh
    const config = vscode.workspace.getConfiguration('odScanner');
    if (config.get('autoRefresh', true)) {
        const intervalSec = config.get('refreshInterval', 60);
        refreshTimer = setInterval(() => performScan(), intervalSec * 1000);
    }
    // Re-register when configuration changes
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('odScanner')) {
            restartAutoRefresh(context);
        }
    }));
}
function deactivate() {
    if (refreshTimer) {
        clearInterval(refreshTimer);
        refreshTimer = undefined;
    }
    statusBarController?.dispose();
    outputChannel?.dispose();
}
async function performScan() {
    statusBarController?.setScanning();
    try {
        const agents = await scannerBridge.scan();
        agentService.update(agents);
        statusBarController?.refresh();
    }
    catch (err) {
        const message = err instanceof types_1.ScannerError ? err.message : String(err);
        statusBarController?.setError(message);
        outputChannel.appendLine(`[Scan Error] ${message}`);
    }
}
function restartAutoRefresh(context) {
    if (refreshTimer) {
        clearInterval(refreshTimer);
        refreshTimer = undefined;
    }
    const config = vscode.workspace.getConfiguration('odScanner');
    if (config.get('autoRefresh', true)) {
        const intervalSec = config.get('refreshInterval', 60);
        refreshTimer = setInterval(() => performScan(), intervalSec * 1000);
    }
}
//# sourceMappingURL=extension.js.map