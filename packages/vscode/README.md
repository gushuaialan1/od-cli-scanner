# OD Scanner — VS Code Extension

Detect & launch local AI coding agents inside VS Code — one click, zero friction.

## Features

- **Status Bar Indicator**: Shows the number of available AI agents in the status bar.
- **One-Click Launch**: Click the status bar to see available agents and launch them in a terminal.
- **Command Palette**: Use `AI Agent: Launch...` from the command palette.
- **Auto Detection**: Automatically detects installed agents on startup.

## Requirements

- [od-cli-scanner](https://github.com/gushuaialan1/od-cli-scanner) (`od-scan`) must be installed and available in your PATH.
- VS Code >= 1.85

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `odScanner.binaryPath` | `""` | Custom path to `od-scan` binary. |
| `odScanner.defaultAgent` | `""` | Default agent to launch. |
| `odScanner.autoRefresh` | `true` | Auto-refresh agent list. |
| `odScanner.refreshInterval` | `60` | Refresh interval in seconds. |
| `odScanner.showUnavailable` | `false` | Show unavailable agents in menu. |

## Usage

1. Install the extension.
2. The status bar will show the number of available AI agents (e.g., `🤖 3`).
3. Click the status bar to select and launch an agent.
4. Or use `Cmd+Shift+P` → `AI Agent: Launch...`.

## Development

```bash
cd packages/vscode
npm install
npm run compile
# Press F5 in VS Code to run the extension in a new Extension Development Host window.
```
