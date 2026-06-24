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
exports.CommandController = void 0;
const vscode = __importStar(require("vscode"));
class CommandController {
    context;
    agentService;
    terminalLauncher;
    scannerBridge;
    constructor(context, agentService, terminalLauncher, scannerBridge) {
        this.context = context;
        this.agentService = agentService;
        this.terminalLauncher = terminalLauncher;
        this.scannerBridge = scannerBridge;
        this.registerCommands();
    }
    registerCommands() {
        const launchCmd = vscode.commands.registerCommand('odScanner.launchAgent', () => this.launchAgent());
        const refreshCmd = vscode.commands.registerCommand('odScanner.refreshAgents', () => this.refreshAgents());
        const settingsCmd = vscode.commands.registerCommand('odScanner.openSettings', () => this.openSettings());
        this.context.subscriptions.push(launchCmd, refreshCmd, settingsCmd);
    }
    async launchAgent() {
        const available = this.agentService.getAvailable();
        if (available.length === 0) {
            const action = await vscode.window.showWarningMessage('No AI agents detected.', 'Refresh', 'Open Settings');
            if (action === 'Refresh') {
                await this.refreshAgents();
            }
            else if (action === 'Open Settings') {
                await this.openSettings();
            }
            return;
        }
        const items = available.map((agent) => ({
            label: agent.name,
            description: agent.version || '',
            detail: agent.path || '',
            agent,
        }));
        const picked = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select an AI agent to launch',
        });
        if (!picked) {
            return;
        }
        const prompt = await vscode.window.showInputBox({
            placeHolder: 'Optional prompt (e.g., "Refactor this module")',
            prompt: 'Enter a prompt to pass to the agent, or leave blank',
        });
        this.terminalLauncher.spawn(picked.agent, prompt || undefined);
    }
    async refreshAgents() {
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Window,
            title: 'Scanning AI agents...',
            cancellable: false,
        }, async () => {
            try {
                const agents = await this.scannerBridge.scan();
                this.agentService.update(agents);
                const available = this.agentService.getAvailable();
                vscode.window.showInformationMessage(`Found ${available.length} AI agent${available.length !== 1 ? 's' : ''}.`);
            }
            catch (err) {
                vscode.window.showErrorMessage(`Agent scan failed: ${err.message}`);
            }
        });
    }
    async openSettings() {
        await vscode.commands.executeCommand('workbench.action.openSettings', 'odScanner');
    }
}
exports.CommandController = CommandController;
//# sourceMappingURL=commandController.js.map