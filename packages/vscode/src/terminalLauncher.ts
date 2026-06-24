import * as vscode from 'vscode';
import { DetectedAgent } from './types';

export class TerminalLauncher {
  spawn(agent: DetectedAgent, prompt?: string): void {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
    const cwd = workspaceRoot || process.cwd();
    const command = prompt ? `${agent.bin} "${prompt.replace(/"/g, '\\"')}"` : agent.bin;

    const terminal = vscode.window.createTerminal({
      name: `AI: ${agent.name}`,
      cwd,
    });

    terminal.sendText(command);
    terminal.show();
  }
}
