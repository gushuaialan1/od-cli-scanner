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
exports.AgentTreeItem = exports.AgentTreeProvider = void 0;
const vscode = __importStar(require("vscode"));
class AgentTreeProvider {
    agentService;
    terminalLauncher;
    _onDidChangeTreeData = new vscode.EventEmitter();
    onDidChangeTreeData = this._onDidChangeTreeData.event;
    constructor(agentService, terminalLauncher) {
        this.agentService = agentService;
        this.terminalLauncher = terminalLauncher;
        this.agentService.onChange(() => this._onDidChangeTreeData.fire());
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (element) {
            return [];
        }
        const folders = vscode.workspace.workspaceFolders;
        if (!folders || folders.length === 0) {
            const agents = this.agentService.getAll();
            if (agents.length === 0) {
                return [
                    new AgentTreeItem('No agents detected', vscode.TreeItemCollapsibleState.None, { tooltip: 'Run a scan to detect agents', icon: 'warning' }),
                ];
            }
            return this.buildAgentItems(agents);
        }
        // Multi-workspace: group by folder
        if (folders.length === 1) {
            const agents = this.agentService.getAll();
            return this.buildAgentItems(agents);
        }
        // Multiple workspace folders: create folder nodes
        return folders.map((folder) => {
            const label = folder.name;
            const item = new AgentTreeItem(label, vscode.TreeItemCollapsibleState.Collapsed, {
                tooltip: `Workspace: ${folder.uri.fsPath}`,
                icon: 'folder',
            });
            item.id = `folder:${folder.uri.fsPath}`;
            return item;
        });
    }
    buildAgentItems(agents) {
        if (agents.length === 0) {
            return [
                new AgentTreeItem('No agents detected', vscode.TreeItemCollapsibleState.None, { tooltip: 'Run a scan to detect agents', icon: 'warning' }),
            ];
        }
        return agents.map((agent) => {
            const status = agent.available
                ? agent.auth_status === 'authenticated'
                    ? '$(check) Available'
                    : agent.auth_status === 'unauthenticated'
                        ? '$(lock) Unauthenticated'
                        : '$(check) Available'
                : '$(debug-disconnect) Unavailable';
            const label = `${agent.name}  ${status}`;
            const description = agent.version || '';
            const tooltip = new vscode.MarkdownString();
            tooltip.appendMarkdown(`**${agent.name}**\n\n`);
            tooltip.appendMarkdown(`- **ID**: ${agent.id}\n`);
            tooltip.appendMarkdown(`- **Path**: ${agent.path || 'N/A'}\n`);
            tooltip.appendMarkdown(`- **Version**: ${agent.version || 'N/A'}\n`);
            tooltip.appendMarkdown(`- **Available**: ${agent.available ? 'Yes' : 'No'}\n`);
            tooltip.appendMarkdown(`- **Auth**: ${agent.auth_status || 'N/A'}\n`);
            if (agent.models && agent.models.length > 0) {
                tooltip.appendMarkdown(`- **Models**: ${agent.models.map((m) => m.label || m.id).join(', ')}\n`);
            }
            if (agent.capabilities && agent.capabilities.length > 0) {
                tooltip.appendMarkdown(`- **Capabilities**: ${agent.capabilities.join(', ')}\n`);
            }
            tooltip.isTrusted = true;
            const item = new AgentTreeItem(label, vscode.TreeItemCollapsibleState.None, {
                description,
                tooltip,
                agent,
                icon: agent.available ? 'play-circle' : 'circle-slash',
                contextValue: agent.available ? 'availableAgent' : 'unavailableAgent',
            });
            if (agent.available) {
                item.command = {
                    command: 'odScanner.launchAgentById',
                    title: 'Launch Agent',
                    arguments: [agent.id],
                };
            }
            return item;
        });
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
}
exports.AgentTreeProvider = AgentTreeProvider;
class AgentTreeItem extends vscode.TreeItem {
    constructor(label, collapsibleState, options = {}) {
        super(label, collapsibleState);
        this.description = options.description;
        this.tooltip = options.tooltip;
        this.iconPath = options.icon
            ? new vscode.ThemeIcon(options.icon)
            : undefined;
        this.contextValue = options.contextValue;
    }
}
exports.AgentTreeItem = AgentTreeItem;
//# sourceMappingURL=agentTreeProvider.js.map