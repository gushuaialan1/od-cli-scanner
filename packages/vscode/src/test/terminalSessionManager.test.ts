import { describe, it, expect, beforeEach, vi } from 'vitest';
import { DetectedAgent } from '../types';

// Mock node-pty so no real PTY processes are spawned during tests.
const writeMock = vi.fn();
const resizeMock = vi.fn();
const killMock = vi.fn();
let dataHandler: ((data: string) => void) | undefined;

vi.mock('node-pty', () => ({
  spawn: vi.fn(() => ({
    write: writeMock,
    resize: resizeMock,
    kill: killMock,
    onData: (cb: (data: string) => void) => {
      dataHandler = cb;
    },
  })),
}));

import { TerminalSessionManager } from '../terminalSessionManager';

function makeAgent(id: string, modelId?: string): DetectedAgent {
  return {
    id,
    name: id,
    bin: `/usr/bin/${id}`,
    available: true,
    models: modelId ? [{ id: modelId, label: modelId }] : undefined,
  };
}

describe('TerminalSessionManager', () => {
  let manager: TerminalSessionManager;

  beforeEach(() => {
    vi.clearAllMocks();
    dataHandler = undefined;
    manager = new TerminalSessionManager();
  });

  describe('spawn()', () => {
    it('returns a session id and tracks the session', () => {
      const id = manager.spawn(makeAgent('claude'));
      expect(id).toMatch(/^session-/);
      expect(manager.getSessionIds()).toEqual([id]);
      expect(manager.getSessionCount()).toBe(1);
    });

    it('sends the agent bin command to the pty', () => {
      manager.spawn(makeAgent('claude'));
      expect(writeMock).toHaveBeenCalledWith('/usr/bin/claude\n');
    });

    it('includes the first model as --model arg when present', () => {
      manager.spawn(makeAgent('kimi', 'k2'));
      expect(writeMock).toHaveBeenCalledWith('/usr/bin/kimi --model k2\n');
    });

    it('throws when the 4-terminal limit is reached', () => {
      for (let i = 0; i < 4; i++) {
        manager.spawn(makeAgent(`a${i}`));
      }
      expect(() => manager.spawn(makeAgent('one-too-many'))).toThrow(
        /Maximum 4 terminals/
      );
    });

    it('notifies session-change callbacks on spawn', () => {
      let calls = 0;
      manager.onSessionChange(() => calls++);
      manager.spawn(makeAgent('a'));
      expect(calls).toBeGreaterThanOrEqual(1);
    });
  });

  describe('write() / resize()', () => {
    it('write forwards data to the pty', () => {
      const id = manager.spawn(makeAgent('a'));
      writeMock.mockClear();
      manager.write(id, 'hello');
      expect(writeMock).toHaveBeenCalledWith('hello');
    });

    it('resize forwards dimensions to the pty', () => {
      const id = manager.spawn(makeAgent('a'));
      manager.resize(id, 120, 40);
      expect(resizeMock).toHaveBeenCalledWith(120, 40);
      const session = manager.getSessions().find((s) => s.id === id);
      expect(session?.cols).toBe(120);
      expect(session?.rows).toBe(40);
    });

    it('ignores unknown session ids without throwing', () => {
      expect(() => manager.write('nope', 'x')).not.toThrow();
      expect(() => manager.resize('nope', 1, 1)).not.toThrow();
      expect(() => manager.kill('nope')).not.toThrow();
    });
  });

  describe('onData()', () => {
    it('routes pty output to the registered callback', () => {
      const id = manager.spawn(makeAgent('a'));
      const received: string[] = [];
      manager.onData(id, (d) => received.push(d));
      dataHandler?.('chunk-1');
      expect(received).toEqual(['chunk-1']);
    });
  });

  describe('kill() / dispose()', () => {
    it('kill removes the session and kills the pty', () => {
      const id = manager.spawn(makeAgent('a'));
      manager.kill(id);
      expect(killMock).toHaveBeenCalled();
      expect(manager.getSessionCount()).toBe(0);
      expect(manager.getSessionIds()).toEqual([]);
    });

    it('kill frees up capacity under the 4-terminal limit', () => {
      const ids: string[] = [];
      for (let i = 0; i < 4; i++) {
        ids.push(manager.spawn(makeAgent(`a${i}`)));
      }
      manager.kill(ids[0]);
      expect(() => manager.spawn(makeAgent('new'))).not.toThrow();
    });

    it('dispose kills all ptys and clears sessions', () => {
      manager.spawn(makeAgent('a'));
      manager.spawn(makeAgent('b'));
      manager.dispose();
      expect(killMock).toHaveBeenCalledTimes(2);
      expect(manager.getSessionCount()).toBe(0);
    });
  });
});
