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
exports.StatusBarController = void 0;
const vscode = __importStar(require("vscode"));
class StatusBarController {
    context;
    agentService;
    terminalLauncher;
    scannerBridge;
    statusBarItem;
    disposables = [];
    nextRefreshTime;
    constructor(context, agentService, terminalLauncher, scannerBridge) {
        this.context = context;
        this.agentService = agentService;
        this.terminalLauncher = terminalLauncher;
        this.scannerBridge = scannerBridge;
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
        this.statusBarItem.command = 'odScanner.statusBarClicked';
        this.context.subscriptions.push(this.statusBarItem);
        const clickCmd = vscode.commands.registerCommand('odScanner.statusBarClicked', () => this.showMenu());
        this.context.subscriptions.push(clickCmd);
        this.disposables.push({ dispose: this.agentService.onChange(() => this.refresh()) });
    }
    setNextRefreshTime(timestamp) {
        this.nextRefreshTime = timestamp;
    }
    refresh() {
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
        }
        else {
            this.statusBarItem.text = `$(robot) ${count}`;
            this.statusBarItem.tooltip = tooltip + `${count} AI agent${count > 1 ? 's' : ''} available. Click to launch.`;
            this.statusBarItem.command = 'odScanner.statusBarClicked';
        }
        this.statusBarItem.show();
    }
    setScanning() {
        this.statusBarItem.text = '$(robot) ...';
        this.statusBarItem.tooltip = 'Scanning for AI agents...';
        this.statusBarItem.show();
    }
    setError(message) {
        this.statusBarItem.text = '$(robot) !';
        this.statusBarItem.tooltip = `Scanner error: ${message}`;
        this.statusBarItem.show();
    }
    async showMenu() {
        const available = this.agentService.getAvailable();
        const config = vscode.workspace.getConfiguration('odScanner');
        const showUnavailable = config.get('showUnavailable', false);
        if (available.length === 0) {
            const choice = await vscode.window.showQuickPick([
                { label: '$(refresh) Refresh Agents', action: 'refresh' },
                { label: '$(gear) Open Settings', action: 'settings' },
                { label: '$(book) View README', action: 'readme' },
            ], { placeHolder: 'No AI agents detected. What would you like to do?' });
            if (!choice) {
                return;
            }
            if (choice.action === 'refresh') {
                await vscode.commands.executeCommand('odScanner.refreshAgents');
            }
            else if (choice.action === 'settings') {
                await vscode.commands.executeCommand('odScanner.openSettings');
            }
            else if (choice.action === 'readme') {
                await vscode.env.openExternal(vscode.Uri.parse('https://github.com/gushuaialan1/od-cli-scanner#readme'));
            }
            return;
        }
        const sortedAgents = this.agentService.getRecentAgents();
        const items = sortedAgents.map((agent) => ({
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
                    agent: undefined,
                });
                for (const agent of unavailable) {
                    items.push({
                        label: `$(debug-disconnect) ${agent.name}`,
                        description: 'Not installed',
                        detail: agent.install_url || '',
                        agent: undefined,
                    });
                }
            }
        }
        items.push({
            label: '',
            description: '',
            detail: '---',
            agent: undefined,
        });
        items.push({
            label: '$(refresh) Refresh Agents',
            description: '',
            detail: '',
            agent: undefined,
            action: 'refresh',
        });
        items.push({
            label: '$(gear) Settings...',
            description: '',
            detail: '',
            agent: undefined,
            action: 'settings',
        });
        const picked = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select an AI agent to launch',
        });
        if (!picked) {
            return;
        }
        if (picked.action === 'refresh') {
            await vscode.commands.executeCommand('odScanner.refreshAgents');
            return;
        }
        if (picked.action === 'settings') {
            await vscode.commands.executeCommand('odScanner.openSettings');
            return;
        }
        if (picked.agent) {
            const selectedModel = await this.pickModelIfNeeded(picked.agent);
            this.terminalLauncher.spawn(picked.agent, undefined, selectedModel);
            this.agentService.recordUsage(picked.agent.id);
        }
    }
    async pickModelIfNeeded(agent) {
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
    dispose() {
        this.statusBarItem.dispose();
        for (const d of this.disposables) {
            d.dispose();
        }
    }
}
exports.StatusBarController = StatusBarController;
//# sourceMappingURL=statusBarController.js.map