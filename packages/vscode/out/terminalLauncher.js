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
exports.TerminalLauncher = void 0;
const vscode = __importStar(require("vscode"));
class TerminalLauncher {
    spawn(agent, prompt, model) {
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
    spawnWithArgs(agent, args, model, workspaceFolder) {
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
exports.TerminalLauncher = TerminalLauncher;
//# sourceMappingURL=terminalLauncher.js.map