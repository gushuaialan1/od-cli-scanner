# OD Scanner — VS Code Extension

Detect & launch local AI coding agents inside VS Code — one click, zero friction.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Icon placeholder** — add a 128×128 PNG icon to `images/icon.png` and reference it in `package.json`.

## Features

- **Status Bar Indicator**: Shows the number of available AI agents in the status bar.
- **One-Click Launch**: Click the status bar to see available agents and launch them in a terminal.
- **Command Palette**: Use `AI Agent: Launch...` from the command palette.
- **Auto Detection**: Automatically detects installed agents on startup.
- **Chat Terminals**: Interactive terminal sessions with detected agents (via Chat View).
- **Agent Tree View**: Browse detected agents in the Explorer sidebar.
- **Context Menu**: Right-click files and "Open with AI Agent" for quick file analysis.
- **Pause/Resume Auto Refresh**: Control automatic agent detection polling.

## Screenshots

<!-- Add screenshots to images/ folder and reference below -->
<!-- ![Screenshot 1](images/screenshot-1.png) -->

## Requirements

- [od-cli-scanner](https://github.com/gushuaialan1/od-cli-scanner) (`od-scan`) must be installed and available in your PATH.
- VS Code >= 1.80

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `odScanner.binaryPath` | `""` | Custom path to `od-scan` binary. |
| `odScanner.defaultAgent` | `""` | Default agent to launch. |
| `odScanner.launchArgs` | `[]` | Global arguments passed to every agent launch. |
| `odScanner.autoRefresh` | `true` | Auto-refresh agent list. |
| `odScanner.refreshInterval` | `60` | Refresh interval in seconds. |
| `odScanner.showUnavailable` | `false` | Show unavailable agents in menu. |

## Usage

1. Install the extension.
2. The status bar will show the number of available AI agents (e.g., `🤖 3`).
3. Click the status bar to select and launch an agent.
4. Or use `Cmd+Shift+P` → `AI Agent: Launch...`.

## Configuration

All settings are accessible via VS Code Settings UI (`odScanner.*`) or `settings.json`:

```json
{
  "odScanner.binaryPath": "/usr/local/bin/od-scan",
  "odScanner.autoRefresh": true,
  "odScanner.refreshInterval": 30
}
```

## Development

```bash
cd packages/vscode
npm install
npm run compile
# Press F5 in VS Code to run the extension in a new Extension Development Host window.
```

## Publishing

To package the extension for the Marketplace:

```bash
npm install -g vsce
vsce package
```

## License

MIT
