# od-cli-scanner Implementation Plan — 2026-07-02

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Complete M6 user choice flow (detect → prompt → add) and prepare M4 Marketplace release.

**Architecture:** 
- M6: Add a "new agents detected" notification in the extension that triggers a quick-pick UI. User chooses which to add. Persist to VS Code global state.
- M4: Ensure extension packaging is clean for Marketplace submission.

**Tech Stack:** TypeScript, VS Code Extension API, Rust, npm

---

## Phase 1: M6 — New Agent Detection UX Flow

### Task 1: Add "New Agents Detected" notification system

**Objective:** After scan completes, compare detected agents against registered ones and show notification for any new ones.

**Files:**
- Modify: `packages/vscode/src/agentService.ts` — add `getNewAgents()` method
- Modify: `packages/vscode/src/extension.ts` — show notification after scan

**Step 1: Add new agent detection to AgentService**

In `agentService.ts`, add a method that tracks which agents the user has already seen/registered:

```typescript
// In AgentService class
private readonly SEEN_AGENTS_KEY = 'odScanner.seenAgents';

bindContext(context: vscode.ExtensionContext): void {
  this.globalState = context.globalState;
  this.recentIds = context.globalState.get<string[]>(RECENT_AGENTS_KEY, []);
  // Initialize seen agents from global state
  this.seenAgents = context.globalState.get<string[]>(this.SEEN_AGENTS_KEY, []);
}

/**
 * Get agents that were detected but not yet seen/registered by the user.
 * @param allDetected - All agents from the latest scan
 * @returns Agents that are new (not in seenAgents list)
 */
getNewAgents(allDetected: DetectedAgent[]): DetectedAgent[] {
  return allDetected.filter(a => !this.seenAgents.includes(a.id));
}

/**
 * Mark agents as seen. Called when user dismisses notification or adds an agent.
 */
markAsSeen(agentIds: string[]): void {
  for (const id of agentIds) {
    if (!this.seenAgents.includes(id)) {
      this.seenAgents.push(id);
    }
  }
  if (this.globalState) {
    this.globalState.update(this.SEEN_AGENTS_KEY, this.seenAgents);
  }
}
```

**Step 2: Show notification in extension.ts**

After `performScan()`, check for new agents and show a notification:

```typescript
// In performScan(), after agentService.update(agents):
const newAgents = agentService.getNewAgents(agents);
if (newAgents.length > 0) {
  const agentNames = newAgents.map(a => a.name).join(', ');
  vscode.window.showInformationMessage(
    `New agents detected: ${agentNames}`,
    'View & Add'
  ).then(selection => {
    if (selection === 'View & Add') {
      vscode.commands.executeCommand('odScanner.showNewAgents');
    }
  });
  agentService.markAsSeen(newAgents.map(a => a.id));
}
```

**Step 3: Register the command**

In `extension.ts`, register the command handler:

```typescript
context.subscriptions.push(
  vscode.commands.registerCommand('odScanner.showNewAgents', () => {
    const newAgents = agentService.getNewAgents(agentService.getAll());
    // Show quick pick to let user choose which to add
    vscode.window.showQuickPick(
      newAgents.map(a => ({
        label: a.name,
        description: a.bin,
        detail: a.version || 'No version detected',
        agent: a,
      })),
      { placeHolder: 'Select agents to add' }
    ).then(selected => {
      if (selected) {
        // Mark as seen
        agentService.markAsSeen(selected.agent ? [selected.agent.id] : []);
      }
    });
  })
);
```

**Step 4: Run compile and verify**

```bash
cd ~/projects/od-cli-scanner/packages/vscode && npm run compile
```

Expected: Clean compile, no errors.

**Step 5: Commit**

```bash
cd ~/projects/od-cli-scanner
git add packages/vscode/src/agentService.ts packages/vscode/src/extension.ts
git commit -m "feat(vscode): M6 — new agent detection notification and quick-pick flow"
```

---

### Task 2: Add agent persistence and config

**Objective:** When user adds a new agent, persist it so it survives extension reloads.

**Files:**
- Modify: `packages/vscode/src/agentService.ts` — add `addCustomAgent()` method
- Modify: `packages/vscode/src/extension.ts` — wire up the add button

**Step 1: Add custom agent support**

```typescript
// In AgentService
private readonly CUSTOM_AGENTS_KEY = 'odScanner.customAgents';

bindContext(context: vscode.ExtensionContext): void {
  // ... existing init ...
  this.customAgents = context.globalState.get<string[]>(this.CUSTOM_AGENTS_KEY, []);
}

/**
 * Add a custom agent to the user's persisted list.
 */
addCustomAgent(agentId: string): void {
  if (!this.customAgents.includes(agentId)) {
    this.customAgents.push(agentId);
    if (this.globalState) {
      this.globalState.update(this.CUSTOM_AGENTS_KEY, this.customAgents);
    }
  }
}

/**
 * Check if an agent is a custom (user-added) one.
 */
isCustomAgent(agentId: string): boolean {
  return this.customAgents.includes(agentId);
}
```

**Step 2: Wire up in extension.ts**

When user selects an agent from the new-agents quick-pick, call `addCustomAgent`:

```typescript
// In the showNewAgents command handler:
if (selected?.agent) {
  agentService.addCustomAgent(selected.agent.id);
  agentService.markAsSeen([selected.agent.id]);
  vscode.window.showInformationMessage(`Added ${selected.agent.name}`);
}
```

**Step 3: Run compile and verify**

```bash
cd ~/projects/od-cli-scanner/packages/vscode && npm run compile
```

**Step 4: Commit**

```bash
cd ~/projects/od-cli-scanner
git add packages/vscode/src/agentService.ts packages/vscode/src/extension.ts
git commit -m "feat(vscode): M6 — custom agent persistence"
```

---

## Phase 2: M4 — Marketplace Preparation

### Task 3: Fix packaging and prepare for release

**Objective:** Ensure the VS Code extension is ready for Marketplace submission.

**Files:**
- Check: `packages/vscode/package.json` — review publisher, icon, readme, etc.
- Check: `.vscodeignore` — ensure unnecessary files excluded

**Step 1: Review package.json for Marketplace requirements**

Check these fields exist and are correct:
- `publisher`: "gushuaialan1"
- `displayName`: "OD Scanner"
- `description`: Bilingual description
- `icon`: Optional but recommended
- `repository`: URL to GitHub repo
- `categories`: ["Programming Languages", "Other"]
- `engines.vscode`: "^1.80.0" or compatible range
- `activationEvents`: ["onStartupFinished"]

**Step 2: Create .vscodeignore**

```
.git/
.hermes/
.vscode/
out/
node_modules/
*.md
!README.md
Cargo.toml
crates/*/Cargo.toml
```

**Step 3: Run VSCE package check**

```bash
cd ~/projects/od-cli-scanner/packages/vscode
npx vsce package --dry-run 2>&1 || echo "vsce not installed, checking manually"
```

**Step 4: Commit any packaging fixes**

```bash
cd ~/projects/od-cli-scanner
git add packages/vscode/package.json packages/vscode/.vscodeignore
git commit -m "chore(vscode): M4 — marketplace packaging preparation"
```

---

## Phase 3: Rust Core Updates

### Task 4: Add streaming output handling for TerminalSessionManager

**Objective:** Fix the current issue where PTY output is not forwarded to the webview.

**Files:**
- Modify: `packages/vscode/src/terminalSessionManager.ts` — add `onData` forwarding
- Modify: `packages/vscode/src/chatTerminalPanel.ts` — handle data messages

**Step 1: Add data forwarding in TerminalSessionManager**

```typescript
// In spawn(), after process.onData:
ptyProcess.onOutput((data: string, isStdout: boolean) => {
  this.notify();
});
```

Wait — node-pty IPty uses `onData` not `onOutput`. The current code already has:
```typescript
ptyProcess.onData(() => {
  this.notify();
});
```

But this doesn't forward the actual data to the webview. We need to capture and forward it.

**Step 2: Modify TerminalSessionManager to expose data events**

```typescript
// Add callback type
private onDataCallbacks: Map<string, (data: string) => void> = new Map();

/**
 * Register a callback for PTY output data.
 */
onData(sessionId: string, callback: (data: string) => void): void {
  this.onDataCallbacks.set(sessionId, callback);
}

// In spawn(), modify onData handler:
ptyProcess.onData((data: string) => {
  const callback = this.onDataCallbacks.get(id);
  if (callback) {
    callback(data);
  }
  this.notify();
});
```

**Step 3: Wire up in ChatTerminalPanel**

```typescript
// In handleSpawn, after getting sessionId:
this.sessionManager.onData(sessionId, (data: string) => {
  // Forward to webview
  if (this.webviewView) {
    this.webviewView.webview.postMessage({
      type: 'terminalData',
      sessionId,
      data,
    });
  }
});
```

**Step 4: Handle in webview HTML**

```javascript
// In the webview script, add message handler:
if (msg.type === 'terminalData') {
  const entry = terminals.get(msg.sessionId);
  if (entry) {
    entry.term.write(msg.data);
  }
}
```

**Step 5: Compile and test**

```bash
cd ~/projects/od-cli-scanner/packages/vscode && npm run compile
```

**Step 6: Commit**

```bash
cd ~/projects/od-cli-scanner
git add packages/vscode/src/terminalSessionManager.ts packages/vscode/src/chatTerminalPanel.ts
git commit -m "fix(vscode): M5 — forward PTY output to webview terminals"
```

---

## Summary of Tasks

| Task | Description | Priority | Est. Time |
|------|-------------|----------|-----------|
| 1 | New agent detection notification | High | 15 min |
| 2 | Custom agent persistence | High | 10 min |
| 3 | Marketplace packaging prep | Medium | 20 min |
| 4 | PTY output forwarding (critical bug fix) | Critical | 15 min |

**Total estimated time: ~60 minutes**

All tasks are independent and can be dispatched as parallel subagents where possible.
