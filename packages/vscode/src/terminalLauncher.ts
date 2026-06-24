import * as vscode from 'vscode';
import { DetectedAgent } from './types';

export class TerminalLauncher {
  spawn(agent: DetectedAgent, prompt?: string, model?: string): void {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
    const cwd = workspaceRoot || process.cwd();
    let command = agent.bin;
    if (model) {
      command += ` --model ${model}`;
    }
    if (prompt) {
      command += ` "${prompt.replace(/"/g, '\\"')}"`;
    }

    const terminal = vscode.window.createTerminal({
      name: `AI: ${agent.name}`,
      cwd,
    });

    terminal.sendText(command);
    terminal.show();
  }

  spawnWithArgs(agent: DetectedAgent, args: string[], model?: string, workspaceFolder?: vscode.WorkspaceFolder): void {
    const cwd = workspaceFolder?.uri.fsPath ?? vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath ?? process.cwd();
    let command = agent.bin;
    if (model) {
      command += ` --model ${model}`;
    }
    const escapedArgs = args.map((arg) => {
      if (arg.includes(' ') || arg.includes('"')) {
        return `"${arg.replace(/"/g, '\\"')}"`;
      }
      return arg;
    });
    if (escapedArgs.length > 0) {
      command += ` ${escapedArgs.join(' ')}`;
    }

    const terminal = vscode.window.createTerminal({
      name: `AI: ${agent.name}`,
      cwd,
    });

    terminal.sendText(command);
    terminal.show();
  }
}
