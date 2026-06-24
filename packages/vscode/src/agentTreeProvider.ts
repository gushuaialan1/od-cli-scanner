import * as vscode from 'vscode';
import { AgentService } from './agentService';
import { TerminalLauncher } from './terminalLauncher';
import { DetectedAgent } from './types';

export class AgentTreeProvider implements vscode.TreeDataProvider<AgentTreeItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<AgentTreeItem | undefined | void> =
    new vscode.EventEmitter<AgentTreeItem | undefined | void>();
  readonly onDidChangeTreeData: vscode.Event<AgentTreeItem | undefined | void> =
    this._onDidChangeTreeData.event;

  constructor(
    private agentService: AgentService,
    private terminalLauncher: TerminalLauncher
  ) {
    this.agentService.onChange(() => this._onDidChangeTreeData.fire());
  }

  getTreeItem(element: AgentTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: AgentTreeItem): AgentTreeItem[] {
    if (element) {
      return [];
    }

    const folders = vscode.workspace.workspaceFolders;
    if (!folders || folders.length === 0) {
      const agents = this.agentService.getAll();
      if (agents.length === 0) {
        return [
          new AgentTreeItem(
            'No agents detected',
            vscode.TreeItemCollapsibleState.None,
            { tooltip: 'Run a scan to detect agents', icon: 'warning' }
          ),
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
      const item = new AgentTreeItem(
        label,
        vscode.TreeItemCollapsibleState.Collapsed,
        {
          tooltip: `Workspace: ${folder.uri.fsPath}`,
          icon: 'folder',
        }
      );
      item.id = `folder:${folder.uri.fsPath}`;
      return item;
    });
  }

  private buildAgentItems(agents: DetectedAgent[]): AgentTreeItem[] {
    if (agents.length === 0) {
      return [
        new AgentTreeItem(
          'No agents detected',
          vscode.TreeItemCollapsibleState.None,
          { tooltip: 'Run a scan to detect agents', icon: 'warning' }
        ),
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

      const item = new AgentTreeItem(
        label,
        vscode.TreeItemCollapsibleState.None,
        {
          description,
          tooltip,
          agent,
          icon: agent.available ? 'play-circle' : 'circle-slash',
          contextValue: agent.available ? 'availableAgent' : 'unavailableAgent',
        }
      );
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

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }
}

interface AgentTreeItemOptions {
  description?: string;
  tooltip?: string | vscode.MarkdownString;
  agent?: DetectedAgent;
  icon?: string;
  contextValue?: string;
}

export class AgentTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    collapsibleState: vscode.TreeItemCollapsibleState,
    options: AgentTreeItemOptions = {}
  ) {
    super(label, collapsibleState);
    this.description = options.description;
    this.tooltip = options.tooltip;
    this.iconPath = options.icon
      ? new vscode.ThemeIcon(options.icon)
      : undefined;
    this.contextValue = options.contextValue;
  }
}
