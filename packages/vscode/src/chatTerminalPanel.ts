import * as vscode from 'vscode';
import { TerminalSessionManager } from './terminalSessionManager';
import { AgentService } from './agentService';

export class ChatTerminalPanel {
  private webviewView: vscode.WebviewView | undefined;
  private sessionManager: TerminalSessionManager;
  private extensionUri: vscode.Uri;
  private agentChangeListener: (() => void) | undefined;

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
        case 'focus':
          this.focusTerminal(message.sessionId);
          break;
      }
    });

    // Listen for agent list changes (scan results updated)
    this.agentChangeListener = this.agentService.onChange(() => {
      this.populateAgentSelector();
    });

    // Populate agent selector
    this.populateAgentSelector();
    this.syncSessionsToWebview();
  }

  private populateAgentSelector(): void {
    if (!this.webviewView) return;
    const available = this.agentService.getAvailable();
    const items = available.map(a =>
      `<option value="${a.id}">${a.name}${a.version ? ' (' + a.version + ')' : ''}</option>`
    ).join('');
    this.webviewView.webview.postMessage({ type: 'populateAgents', agents: items });
  }

  private async handleSpawn(agentId: string): Promise<void> {
    const agent = this.agentService.getById(agentId);
    if (!agent) return;
    try {
      const sessionId = this.sessionManager.spawn(agent);
      // Register data callback to forward PTY output to webview
      this.sessionManager.onData(sessionId, (data: string) => {
        if (this.webviewView) {
          this.webviewView.webview.postMessage({
            type: 'terminalData',
            sessionId,
            data,
          });
        }
      });
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

  private focusTerminal(sessionId: string): void {
    // Focus the webview to receive keyboard events
    if (this.webviewView) {
      this.webviewView.webview.postMessage({ type: 'focus', sessionId });
    }
  }

  private getHtml(): string {
    const termJsUri = this.webviewView!.webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'node_modules', '@xterm', 'xterm', 'lib', 'xterm.js')
    );
    const fitJsUri = this.webviewView!.webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'node_modules', '@xterm', 'addon-fit', 'lib', 'addon-fit.js')
    );
    const termCssUri = this.webviewView!.webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'media', 'xterm.css')
    );

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <link href="${termCssUri}" rel="stylesheet">
  <style>
    html, body { margin: 0; padding: 0; height: 100vh; overflow: hidden; background: #1e1e1e; }
    #toolbar { display: none; gap: 8px; padding: 6px 8px; background: #2d2d2d; align-items: center; border-bottom: 1px solid #3c3c3c; }
    #toolbar.visible { display: flex; }
    #toolbar button { background: #0e639c; color: #fff; border: none; padding: 3px 10px; border-radius: 3px; cursor: pointer; font-size: 11px; }
    #toolbar button:hover { background: #1177bb; }
    #toolbar button:disabled { opacity: 0.4; cursor: not-allowed; }
    #toolbar span.count { color: #888; font-size: 11px; margin-left: auto; }
    #grid { display: grid; height: calc(100vh - 36px); gap: 2px; background: #1e1e1e; padding: 2px; }
    #grid.layout-1x1 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
    #grid.layout-1x2 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
    #grid.layout-2x2 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }
    .terminal-cell { background: #1e1e1e; position: relative; overflow: hidden; }
    .terminal-cell .close-btn {
      position: absolute; top: 2px; right: 2px; z-index: 10;
      background: #c42b1c; color: #fff; border: none;
      width: 16px; height: 16px; border-radius: 2px;
      cursor: pointer; font-size: 9px; line-height: 1;
      display: flex; align-items: center; justify-content: center;
    }
    .terminal-cell .close-btn:hover { background: #d8453a; }
    #empty { display: flex; align-items: center; justify-content: center; height: 100vh; color: #888; font-size: 13px; flex-direction: column; gap: 10px; }
    #empty select, #empty button { padding: 4px 8px; font-size: 12px; }
    #empty select { background: #3c3c3c; color: #ccc; border: 1px solid #555; border-radius: 3px; }
    #empty button { background: #0e639c; color: #fff; border: none; border-radius: 3px; cursor: pointer; }
  </style>
</head>
<body>
  <div id="toolbar">
    <button id="btn-add">+ Add Terminal</button>
    <span class="count" id="session-count">0/4</span>
  </div>
  <div id="grid" class="layout-1x1"></div>
  <div id="empty">
    <p>No terminals open.</p>
    <select id="agent-select"><option value="">Select an agent...</option></select>
    <button id="btn-launch">Launch</button>
  </div>
  <script src="${termJsUri}"></script>
  <script src="${fitJsUri}"></script>
  <script>
    const vscode = acquireVsCodeApi();
    const { Terminal } = window.Terminal;
    const { FitAddon } = window.FitAddon;
    const fitAddon = new FitAddon();
    const terminals = new Map();
    let activeSessionId = null;

    window.addEventListener('message', event => {
      const msg = event.data;
      if (msg.type === 'populateAgents') {
        const sel = document.getElementById('agent-select');
        sel.innerHTML = '<option value="">Select an agent...</option>' + msg.agents;
      } else if (msg.type === 'sessionsUpdated') {
        renderSessions(msg.sessions, msg.layout);
      } else if (msg.type === 'terminalData') {
        const entry = terminals.get(msg.sessionId);
        if (entry) {
          entry.term.write(msg.data);
        }
      } else if (msg.type === 'focus') {
        focusTerminal(msg.sessionId);
      }
    });

    function renderSessions(sessions, layout) {
      const grid = document.getElementById('grid');
      const empty = document.getElementById('empty');
      const toolbar = document.getElementById('toolbar');

      grid.className = 'layout-' + layout;
      grid.innerHTML = '';
      empty.style.display = sessions.length === 0 ? 'flex' : 'none';
      toolbar.classList.toggle('visible', sessions.length > 0);

      sessions.forEach(s => {
        const cell = document.createElement('div');
        cell.className = 'terminal-cell';
        cell.dataset.sessionId = s.id;

        const closeBtn = document.createElement('button');
        closeBtn.className = 'close-btn';
        closeBtn.textContent = '×';
        closeBtn.addEventListener('click', () => {
          vscode.postMessage({ type: 'kill', sessionId: s.id });
        });
        cell.appendChild(closeBtn);

        const term = new Terminal({
          cursorBlink: true,
          theme: { background: '#1e1e1e', foreground: '#cccccc' },
        });
        term.loadAddon(fitAddon);
        term.open(cell);
        terminals.set(s.id, { term, cell });

        term.onBinary(data => {
          vscode.postMessage({ type: 'input', sessionId: s.id, data: data });
        });

        term.onFocus(() => {
          activeSessionId = s.id;
          vscode.postMessage({ type: 'focus', sessionId: s.id });
        });

        setTimeout(() => {
          fitAddon.fit();
          term.focus();
          activeSessionId = s.id;
        }, 50);
      });

      document.getElementById('session-count').textContent = sessions.length + '/4';
    }

    function focusTerminal(sessionId) {
      const entry = terminals.get(sessionId);
      if (entry) {
        entry.term.focus();
        activeSessionId = sessionId;
      }
    }

    document.getElementById('btn-add').addEventListener('click', () => {
      const sel = document.getElementById('agent-select');
      if (sel.value) {
        vscode.postMessage({ type: 'spawn', agentId: sel.value });
      }
    });

    document.getElementById('btn-launch').addEventListener('click', () => {
      const sel = document.getElementById('agent-select');
      if (sel.value) {
        vscode.postMessage({ type: 'spawn', agentId: sel.value });
      }
    });

    // Ctrl+Tab: cycle through terminals
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Tab') {
        e.preventDefault();
        const sessionIds = Array.from(terminals.keys());
        if (sessionIds.length === 0) return;
        let idx = sessionIds.indexOf(activeSessionId);
        idx = (idx + 1) % sessionIds.length;
        activeSessionId = sessionIds[idx];
        const entry = terminals.get(activeSessionId);
        if (entry) {
          entry.term.focus();
          vscode.postMessage({ type: 'focus', sessionId: activeSessionId });
        }
      }
    });
  </script>
</body>
</html>`;
  }

  refresh(): void {
    this.syncSessionsToWebview();
  }

  dispose(): void {
    if (this.agentChangeListener) {
      this.agentChangeListener();
    }
    this.sessionManager.dispose();
  }
}
