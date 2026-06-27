import * as os from 'os';
import * as pty from 'node-pty';
import { DetectedAgent } from './types';

export interface TerminalSession {
  id: string;
  agent: DetectedAgent;
  ptyProcess: pty.IPty;
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

  spawn(agent: DetectedAgent, _workspaceFolder?: string): string {
    if (this.sessions.size >= 4) {
      throw new Error('Maximum 4 terminals allowed');
    }

    const id = `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const shell = os.platform() === 'win32' ? 'cmd.exe' : (process.env.SHELL || '/bin/bash');

    const ptyProcess = pty.spawn(shell, [], {
      name: 'xterm-color',
      cols: 80,
      rows: 30,
      cwd: process.cwd(),
      env: process.env as any,
    });

    // Send agent command to start it
    const modelArg = agent.models?.[0]?.id ? ` --model ${agent.models[0].id}` : '';
    const command = agent.bin + modelArg + '\n';
    ptyProcess.write(command);

    ptyProcess.onData(() => {
      this.notify();
    });

    const session: TerminalSession = { id, agent, ptyProcess, cols: 80, rows: 30 };
    this.sessions.set(id, session);
    this.notify();
    return id;
  }

  write(sessionId: string, data: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.ptyProcess.write(data);
    }
  }

  resize(sessionId: string, cols: number, rows: number): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.ptyProcess.resize(cols, rows);
      session.cols = cols;
      session.rows = rows;
    }
  }

  kill(sessionId: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.ptyProcess.kill();
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
      session.ptyProcess.kill();
    }
    this.sessions.clear();
  }
}
