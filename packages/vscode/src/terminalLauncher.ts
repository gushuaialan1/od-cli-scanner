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

  spawnWithArgs(agent: DetectedAgent, args: string[]): void {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
    const cwd = workspaceRoot || process.cwd();
    const escapedArgs = args.map((arg) => {
      if (arg.includes(' ') || arg.includes('"')) {
        return `"${arg.replace(/"/g, '\\"')}"`;
      }
      return arg;
    });
    const command = escapedArgs.length > 0 ? `${agent.bin} ${escapedArgs.join(' ')}` : agent.bin;

    const terminal = vscode.window.createTerminal({
      name: `AI: ${agent.name}`,
      cwd,
    });

    terminal.sendText(command);
    terminal.show();
  }
}
