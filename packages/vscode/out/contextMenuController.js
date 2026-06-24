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
exports.ContextMenuController = void 0;
const vscode = __importStar(require("vscode"));
class ContextMenuController {
    context;
    agentService;
    terminalLauncher;
    scannerBridge;
    disposables = [];
    constructor(context, agentService, terminalLauncher, scannerBridge) {
        this.context = context;
        this.agentService = agentService;
        this.terminalLauncher = terminalLauncher;
        this.scannerBridge = scannerBridge;
        this.registerCommands();
    }
    registerCommands() {
        const openWithCmd = vscode.commands.registerCommand('odScanner.openWithAgent', (uri, selectedUris) => this.openWithAgent(uri, selectedUris));
        this.context.subscriptions.push(openWithCmd);
    }
    async openWithAgent(uri, selectedUris) {
        const available = this.agentService.getAvailable();
        if (available.length === 0) {
            const action = await vscode.window.showWarningMessage('No AI agents detected.', 'Refresh', 'Open Settings');
            if (action === 'Refresh') {
                await vscode.commands.executeCommand('odScanner.refreshAgents');
            }
            else if (action === 'Open Settings') {
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
        if (!picked) {
            return;
        }
        const prompt = await vscode.window.showInputBox({
            placeHolder: 'Optional prompt (e.g., "Review this file")',
            prompt: 'Enter a prompt to pass to the agent, or leave blank',
        });
        this.launchAgent(picked.agent, contextPaths, prompt || undefined);
    }
    launchAgent(agent, contextPaths, prompt) {
        const config = vscode.workspace.getConfiguration('odScanner');
        const globalArgs = config.get('launchArgs', []);
        const args = [...globalArgs];
        if (contextPaths.length > 0) {
            args.push(...contextPaths);
        }
        if (prompt) {
            args.push(prompt);
        }
        this.terminalLauncher.spawnWithArgs(agent, args);
        this.agentService.recordUsage(agent.id);
    }
    dispose() {
        for (const d of this.disposables) {
            d.dispose();
        }
    }
}
exports.ContextMenuController = ContextMenuController;
//# sourceMappingURL=contextMenuController.js.map