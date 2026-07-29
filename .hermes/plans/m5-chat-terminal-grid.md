# M5: Chat-Style Terminal Grid — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Replace single-terminal spawn with a multi-pane chat-style terminal grid (1-4 agents) in a VS Code WebviewView sidebar panel.

**Architecture:** VS Code `WebviewViewProvider` + `node-pty` (PTY creation) + `@xterm/xterm` (browser rendering) + `@xterm/addon-fit` (resize). The WebviewView lives in the sidebar alongside the existing tree view. Each pane is an independent PTY backed by xterm.js terminal instance.

**Tech Stack:** TypeScript, VS Code Extension API, node-pty@1.0.0, @xterm/xterm@5.5.0, @xterm/addon-fit, HTML/CSS Grid

---

## Task 1: Create ChatTerminalPanel (WebviewViewProvider scaffold)

**Objective:** Scaffold the new WebviewViewProvider that creates the chat-style terminal sidebar panel.

**Files:**
- Create: `packages/vscode/src/chatTerminalPanel.ts`
- Modify: `packages/vscode/src/extension.ts` (register the view)
- Modify: `packages/vscode/package.json` (add `views` entry for chatTerminal)

**Step 1: Write the scaffold**

Create `packages/vscode/src/chatTerminalPanel.ts`:

```typescript
import * as vscode from 'vscode';

export class ChatTerminalPanel {
  private panel: vscode.WebviewView;
  private disposables: vscode.Disposable[] = [];

  constructor(private context: vscode.ExtensionContext) {}

  resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ): void {
    this.panel = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.context.extensionUri],
    };

    webviewView.webview.html = this.getHtml();
  }

  private getHtml(): string {
    const termscriptUri = this.panel.webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, 'media', 'xterm.css')
    );

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <link href="${termscriptUri}" rel="stylesheet">
  <style>
    html, body { margin: 0; padding: 0; height: 100%; overflow: hidden; }
    #terminal-container { width: 100%; height: 100%; }
  </style>
</head>
<body>
  <div id="terminal-container"></div>
  <script>
    const vscode = acquireVsCodeApi();
    const container = document.getElementById('terminal-container');
    // xterm will be loaded via postMessage from extension host
  </script>
</body>
</html>`;
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
```

**Step 2: Register in extension.ts**

In `extension.ts` `activate()`, after existing registrations:

```typescript
// Chat terminal panel
const chatTerminalPanel = new ChatTerminalPanel(context);
const chatView = vscode.window.registerWebviewViewProvider(
  'odScanner.chatTerminal',
  chatTerminalPanel
);
context.subscriptions.push(chatView);
```

**Step 3: Add view contribution to package.json**

Under `contributes.views`, add:

```json
"chatExplorer": [
  {
    "id": "odScanner.chatTerminal",
    "name": "Chat Terminals",
    "when": "odScanner.loaded"
  }
]
```

Also add a `when` clause registration in extension.ts after initial scan:

```typescript
vscode.commands.executeCommand('setContext', 'odScanner.loaded', true);
```

**Step 4: Verify compilation**

Run: `cd packages/vscode && npm run compile`
Expected: No errors, clean compile.

**Step 5: Commit**

```bash
cd ~/projects/od-cli-scanner/packages/vscode
git add src/chatTerminalPanel.ts src/extension.ts package.json
git commit -m "feat(vscode): M5 P1 — scaffold ChatTerminalPanel WebviewView"
```

---

## Task 2: Add xterm.js and CSS media files

**Objective:** Install xterm.js dependencies and add required CSS.

**Files:**
- Install: `@xterm/xterm`, `@xterm/addon-fit`
- Create: `packages/vscode/media/xterm.css`

**Step 1: Install dependencies**

```bash
cd ~/projects/od-cli-scanner/packages/vscode
npm install @xterm/xterm@5.5.0 @xterm/addon-fit@0.10.0 --save
```

**Step 2: Copy xterm.css**

Copy `node_modules/@xterm/xterm/css/xterm.css` to `media/xterm.css`.

**Step 3: Verify**

```bash
cd packages/vscode && npm run compile
```

**Step 4: Commit**

```bash
git add package.json package-lock.json media/xterm.css
git commit -m "feat(vscode): M5 P2 — add xterm.js dependencies and CSS"
```

---

## Task 3: Implement PTY manager (node-pty wrapper)

**Objective:** Create a `TerminalSessionManager` class that manages 1-4 PTY instances, each backing one agent.

**Files:**
- Create: `packages/vscode/src/terminalSessionManager.ts`

**Step 1: Write the manager**

```typescript
import * as os from 'os';
import * as pty from 'node-pty';
import { DetectedAgent } from './types';

export interface TerminalSession {
  id: string;
  agent: DetectedAgent;
  process: pty.IPty;
  cols: number;
  rows: number;
}

export class TerminalSessionManager {
  private sessions: Map<string, TerminalSession> = new Map();
  private onSessionChangeCallbacks: (() => void)[] = [];

  onSessionChange(cb: () => void): void {
    this.onSessionChangeCallbacks.push(cb);
  }

  private notify(): void {
    for (const cb of this.onSessionChangeCallbacks) {
      cb();
    }
  }

  spawn(agent: DetectedAgent, workspaceFolder?: string): string {
    // Check max 4 sessions
    if (this.sessions.size >= 4) {
      throw new Error('Maximum 4 terminals allowed');
    }

    const id = `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const cwd = workspaceFolder || process.cwd();
    const shell = os.platform() === 'win32' ? 'cmd.exe' : (process.env.SHELL || '/bin/bash');

    const process = pty.spawn(shell, [], {
      name: 'xterm-color',
      cols: 80,
      rows: 30,
      cwd,
      env: process.env as any,
    });

    // Send the agent command via stdin
    const modelArg = agent.models?.[0]?.id ? ` --model ${agent.models[0].id}` : '';
    const command = agent.bin + modelArg;

    process.onData((data) => {
      // Forward PTY output to webview
      this.notify();
    });

    const session: TerminalSession = { id, agent, process, cols: 80, rows: 30 };
    this.sessions.set(id, session);
    this.notify();
    return id;
  }

  write(sessionId: string, data: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.process.write(data);
    }
  }

  resize(sessionId: string, cols: number, rows: number): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.process.resize(cols, rows);
      session.cols = cols;
      session.rows = rows;
    }
  }

  kill(sessionId: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.process.kill();
      this.sessions.delete(sessionId);
      this.notify();
    }
  }

  getSessionIds(): string[] {
    return Array.from(this.sessions.keys());
  }

  getSessionCount(): number {
    return this.sessions.size;
  }

  getSessions(): TerminalSession[] {
    return Array.from(this.sessions.values());
  }

  dispose(): void {
    for (const [, session] of this.sessions) {
      session.process.kill();
    }
    this.sessions.clear();
  }
}
```

**Step 2: Verify compilation**

```bash
cd ~/projects/od-cli-scanner/packages/vscode && npm run compile
```

**Step 3: Commit**

```bash
git add src/terminalSessionManager.ts
git commit -m "feat(vscode): M5 P3 — implement TerminalSessionManager with node-pty"
```

---

## Task 4: Wire ChatTerminalPanel with xterm.js rendering

**Objective:** Connect the WebviewView to xterm.js terminals and the PTY manager.

**Files:**
- Modify: `packages/vscode/src/chatTerminalPanel.ts`
- Create: `packages/vscode/media/chat-terminal.html` (externalize HTML for clarity)

**Step 1: Update chatTerminalPanel.ts**

Replace the scaffold with full implementation:

```typescript
import * as vscode from 'vscode';
import { TerminalSessionManager } from './terminalSessionManager';
import { AgentService } from './agentService';

export class ChatTerminalPanel {
  private webviewView: vscode.WebviewView | undefined;
  private sessionManager: TerminalSessionManager;
  private extensionUri: vscode.Uri;

  constructor(
    private context: vscode.ExtensionContext,
    private agentService: AgentService
  ) {
    this.sessionManager = new TerminalSessionManager();
    this.extensionUri = context.extensionUri;
  }

  resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ): void {
    this.webviewView = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.extensionUri],
    };

    webviewView.webview.html = this.getHtml();

    // Listen for messages from webview
    webviewView.webview.onDidReceiveMessage(async (message) => {
      switch (message.type) {
        case 'spawn':
          await this.handleSpawn(message.agentId);
          break;
        case 'input':
          this.sessionManager.write(message.sessionId, message.data);
          break;
        case 'resize':
          this.sessionManager.resize(message.sessionId, message.cols, message.rows);
          break;
        case 'kill':
          this.sessionManager.kill(message.sessionId);
          break;
      }
    });
  }

  private async handleSpawn(agentId: string): Promise<void> {
    const agent = this.agentService.getById(agentId);
    if (!agent) return;
    try {
      const sessionId = this.sessionManager.spawn(agent);
      this.syncSessionsToWebview();
    } catch (err) {
      vscode.window.showErrorMessage(String(err));
    }
  }

  private syncSessionsToWebview(): void {
    if (!this.webviewView) return;
    const sessions = this.sessionManager.getSessions();
    const count = sessions.length;
    this.webviewView.webview.postMessage({
      type: 'sessionsUpdated',
      sessions: sessions.map(s => ({
        id: s.id,
        agentName: s.agent.name,
        agentId: s.agent.id,
        cols: s.cols,
        rows: s.rows,
      })),
      layout: count === 1 ? '1x1' : count === 2 ? '1x2' : '2x2',
    });
  }

  private getHtml(): string {
    const termscriptUri = this.webviewView!.webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'media', 'xterm.css')
    );
    const mediaUri = this.webviewView!.webview.asWebviewUri(this.extensionUri);

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <link href="${termscriptUri}" rel="stylesheet">
  <style>
    html, body { margin: 0; padding: 0; height: 100vh; overflow: hidden; background: #1e1e1e; }
    #toolbar { display: flex; gap: 8px; padding: 8px; background: #2d2d2d; align-items: center; }
    #toolbar button { background: #0e639c; color: #fff; border: none; padding: 4px 12px; border-radius: 3px; cursor: pointer; font-size: 12px; }
    #toolbar button:hover { background: #1177bb; }
    #toolbar button:disabled { opacity: 0.4; cursor: not-allowed; }
    #grid { display: grid; height: calc(100vh - 40px); gap: 2px; background: #1e1e1e; padding: 2px; }
    #grid.layout-1x1 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
    #grid.layout-1x2 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
    #grid.layout-2x2 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }
    .terminal-cell { background: #1e1e1e; position: relative; overflow: hidden; }
    .xterm-screen { padding: 0 !important; }
    #empty { display: flex; align-items: center; justify-content: center; height: 100vh; color: #888; font-size: 14px; flex-direction: column; gap: 12px; }
    #empty select { padding: 4px 8px; background: #3c3c3c; color: #ccc; border: 1px solid #555; border-radius: 3px; }
    #empty button { padding: 4px 16px; background: #0e639c; color: #fff; border: none; border-radius: 3px; cursor: pointer; }
  </style>
</head>
<body>
  <div id="toolbar">
    <button id="btn-add">+ Add Terminal</button>
    <span style="color:#888;font-size:11px;margin-left:auto;" id="session-count">0/4 sessions</span>
  </div>
  <div id="grid" class="layout-1x1"></div>
  <div id="empty">
    <p>No terminals open.</p>
    <select id="agent-select"><option value="">Select an agent...</option></select>
    <button id="btn-launch">Launch</button>
  </div>
  <script src="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/lib/xterm.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10.0/lib/addon-fit.js"></script>
  <script>
    const vscode = acquireVsCodeApi();
    const { Terminal } = XTermModule; // Will be set by postMessage
    const fitAddon = new FitAddon.FitAddon();
    const terminals = new Map();
    let sessionCounter = 0;

    // Listen for messages from extension host
    window.addEventListener('message', event => {
      const msg = event.data;
      if (msg.type === 'sessionsUpdated') {
        renderSessions(msg.sessions, msg.layout);
      }
    });

    function renderSessions(sessions, layout) {
      const grid = document.getElementById('grid');
      const empty = document.getElementById('empty');
      const toolbar = document.getElementById('toolbar');

      grid.className = 'layout-' + layout;
      grid.innerHTML = '';
      empty.style.display = sessions.length === 0 ? 'flex' : 'none';
      toolbar.style.display = sessions.length > 0 ? 'flex' : 'none';

      sessions.forEach(s => {
        const cell = document.createElement('div');
        cell.className = 'terminal-cell';
        cell.dataset.sessionId = s.id;
        const term = new Terminal({
          cursorBlink: true,
          theme: { background: '#1e1e1e', foreground: '#cccccc' },
        });
        term.loadAddon(fitAddon);
        term.open(cell);
        terminals.set(s.id, { term, cell });
        fitAddon.fit();
      });

      document.getElementById('session-count').textContent = sessions.length + '/4 sessions';
    }

    // Add button
    document.getElementById('btn-add').addEventListener('click', () => {
      const select = document.getElementById('agent-select');
      if (select.value) {
        vscode.postMessage({ type: 'spawn', agentId: select.value });
      }
    });

    // Initial setup
    document.getElementById('empty').style.display = 'flex';
    document.getElementById('toolbar').style.display = 'none';
  </script>
</body>
</html>`;
  }

  refresh(): void {
    this.syncSessionsToWebview();
  }

  dispose(): void {
    this.sessionManager.dispose();
  }
}
```

**Step 2: Update extension.ts to wire it up**

```typescript
// Chat terminal panel
const chatTerminalPanel = new ChatTerminalPanel(context, agentService);
const chatView = vscode.window.registerWebviewViewProvider(
  'odScanner.chatTerminal',
  chatTerminalPanel
);
context.subscriptions.push(chatView);
```

**Step 3: Verify compilation**

```bash
cd ~/projects/od-cli-scanner/packages/vscode && npm run compile
```

**Step 4: Commit**

```bash
git add src/chatTerminalPanel.ts src/extension.ts
git commit -m "feat(vscode): M5 P4 — wire ChatTerminalPanel with xterm.js rendering"
```

---

## Task 5: Agent selector and spawn flow

**Objective:** Populate agent dropdown, handle spawn from UI, forward input to PTY.

**Files:**
- Modify: `packages/vscode/src/chatTerminalPanel.ts` (HTML + JS in getHtml)

**Step 1: Add agent selector population**

In `resolveWebviewView`, after setting up message listener:

```typescript
// Populate agent selector
const available = this.agentService.getAvailable();
const items = available.map(a => `<option value="${a.id}">${a.name} (${a.version || 'latest'})</option>`).join('');
webviewView.webview.postMessage({ type: 'populateAgents', agents: items });
```

**Step 2: Add input forwarding**

In the webview HTML, add keydown listener on each terminal:

```javascript
terminals.forEach(({ term, cell }, sessionId) => {
  term.onKeyDown(e => {
    vscode.postMessage({ type: 'input', sessionId, data: e.key });
  });
});
```

**Step 3: Commit**

```bash
git commit -am "feat(vscode): M5 P5 — agent selector and spawn flow"
```

---

## Task 6: Polish — close buttons, resize, styling

**Objective:** Add per-terminal close buttons, proper resize handling, dark theme matching.

**Files:**
- Modify: `packages/vscode/src/chatTerminalPanel.ts`

**Step 1: Add close button per terminal**

Each `.terminal-cell` gets a close button overlay:

```css
.terminal-cell .close-btn {
  position: absolute; top: 2px; right: 2px;
  background: #c42b1c; color: #fff; border: none;
  width: 18px; height: 18px; border-radius: 2px;
  cursor: pointer; font-size: 10px; line-height: 1;
  z-index: 10;
}
```

**Step 2: Handle resize events**

```javascript
window.addEventListener('resize', () => {
  terminals.forEach(({ term }) => fitAddon.fit());
});
```

**Step 3: Commit**

```bash
git commit -am "feat(vscode): M5 P6 — polish close buttons, resize, styling"
```

---

## Verification Steps (after all tasks)

1. `cd packages/vscode && npm run compile` — clean compile
2. Install the .vsix in VS Code
3. Open sidebar → "Chat Terminals" view appears
4. Click "+ Add Terminal" → select agent → launches PTY
5. Verify: 1/2/4 pane layouts work
6. Verify: input flows through, output renders in xterm
7. Verify: close button kills PTY and removes cell
8. `cd ~/projects/od-cli-scanner && cargo test --workspace` — Rust tests still pass

---

## Risks & Tradeoffs

| Risk | Mitigation |
|------|-----------|
| node-pty native module rebuild on install | Already installed, prebuilds exist for linux-x64 |
| xterm.js CDN vs bundled | Bundle xterm.js via esbuild to avoid CDN dependency |
| PTY stdin forwarding for agents that expect interactive mode | Some agents (Claude Code) work fine with piped stdin; test each |
| Memory usage with 4 PTYs | Limit to 4, kill unused sessions, monitor RSS |
| Webview vs TreeView coexistence | Use `viewsContainer` to nest under existing "AI Agents" view |
