import { describe, it, expect, beforeEach } from 'vitest';
import { AgentService } from '../agentService';
import { DetectedAgent } from '../types';

function makeAgent(id: string, available = true, extra: Partial<DetectedAgent> = {}): DetectedAgent {
  return { id, name: id.toUpperCase(), bin: id, available, ...extra };
}

/** Minimal in-memory Memento stand-in. */
function makeMemento(initial: Record<string, unknown> = {}) {
  const store = new Map<string, unknown>(Object.entries(initial));
  return {
    get: <T>(key: string, defaultValue?: T): T =>
      (store.has(key) ? (store.get(key) as T) : (defaultValue as T)),
    update: async (key: string, value: unknown) => {
      store.set(key, value);
    },
    store,
  };
}

function bindService(service: AgentService, memento: ReturnType<typeof makeMemento>) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  service.bindContext({ globalState: memento } as any);
}

describe('AgentService', () => {
  let service: AgentService;

  beforeEach(() => {
    service = new AgentService();
  });

  describe('update() / getAll() / getAvailable()', () => {
    it('returns empty arrays before any update', () => {
      expect(service.getAll()).toEqual([]);
      expect(service.getAvailable()).toEqual([]);
    });

    it('getAll returns all agents; getAvailable filters available=true', () => {
      service.update([makeAgent('a'), makeAgent('b', false), makeAgent('c')]);
      expect(service.getAll().map((a) => a.id)).toEqual(['a', 'b', 'c']);
      expect(service.getAvailable().map((a) => a.id)).toEqual(['a', 'c']);
    });

    it('getAll returns a copy, not the internal array', () => {
      service.update([makeAgent('a')]);
      const all = service.getAll();
      all.push(makeAgent('intruder'));
      expect(service.getAll()).toHaveLength(1);
    });

    it('populates the models map from agent data', () => {
      const models = [{ id: 'm1', label: 'Model 1' }];
      service.update([makeAgent('a', true, { models }), makeAgent('b')]);
      expect(service.getModels('a')).toEqual(models);
      expect(service.getModels('b')).toBeUndefined();
    });

    it('clears stale models on subsequent update', () => {
      service.update([makeAgent('a', true, { models: [{ id: 'm1', label: 'M1' }] })]);
      service.update([makeAgent('a')]);
      expect(service.getModels('a')).toBeUndefined();
    });
  });

  describe('getById()', () => {
    it('finds agents regardless of availability', () => {
      service.update([makeAgent('a'), makeAgent('b', false)]);
      expect(service.getById('b')?.id).toBe('b');
      expect(service.getById('missing')).toBeUndefined();
    });
  });

  describe('recordUsage() / getRecentAgents()', () => {
    beforeEach(() => {
      service.update([makeAgent('a'), makeAgent('b'), makeAgent('c')]);
    });

    it('moves the used agent to the front', () => {
      service.recordUsage('c');
      expect(service.getRecentAgents().map((a) => a.id)[0]).toBe('c');
    });

    it('re-recording an agent keeps it at the front without duplicates', () => {
      service.recordUsage('a');
      service.recordUsage('b');
      service.recordUsage('a');
      expect(service.getRecentAgents().map((a) => a.id)).toEqual(['a', 'b', 'c']);
    });

    it('caps the recent list at 5', () => {
      service.update([
        makeAgent('a'), makeAgent('b'), makeAgent('c'),
        makeAgent('d'), makeAgent('e'), makeAgent('f'),
      ]);
      for (const id of ['a', 'b', 'c', 'd', 'e', 'f']) {
        service.recordUsage(id);
      }
      // f is most recent; 'a' should have been evicted from recent ordering,
      // so 'a' falls back to the tail with the non-recent agents.
      const ordered = service.getRecentAgents().map((x) => x.id);
      expect(ordered.slice(0, 5)).toEqual(['f', 'e', 'd', 'c', 'b']);
      expect(ordered[5]).toBe('a');
    });

    it('recent list only includes available agents', () => {
      service.recordUsage('b');
      service.update([makeAgent('a'), makeAgent('b', false), makeAgent('c')]);
      const ordered = service.getRecentAgents().map((x) => x.id);
      expect(ordered).toEqual(['a', 'c']);
    });

    it('persists recents to globalState when bound', () => {
      const memento = makeMemento();
      bindService(service, memento);
      service.recordUsage('b');
      expect(memento.store.get('odScanner.recentAgents')).toEqual(['b']);
    });

    it('restores recents from globalState on bindContext', () => {
      const memento = makeMemento({ 'odScanner.recentAgents': ['c', 'a'] });
      bindService(service, memento);
      expect(service.getRecentAgents().map((a) => a.id)).toEqual(['c', 'a', 'b']);
    });
  });

  describe('getNewAgents() / markAsSeen()', () => {
    const detected = [makeAgent('a'), makeAgent('b'), makeAgent('c')];

    it('all detected agents are new before any are seen', () => {
      expect(service.getNewAgents(detected).map((a) => a.id)).toEqual(['a', 'b', 'c']);
    });

    it('markAsSeen removes agents from the new set', () => {
      service.markAsSeen(['a', 'c']);
      expect(service.getNewAgents(detected).map((a) => a.id)).toEqual(['b']);
    });

    it('markAsSeen is idempotent', () => {
      service.markAsSeen(['a']);
      service.markAsSeen(['a']);
      const memento = makeMemento({ 'odScanner.seenAgents': ['a'] });
      bindService(service, memento);
      service.markAsSeen(['a']);
      expect(memento.store.get('odScanner.seenAgents')).toEqual(['a']);
    });

    it('restores seen agents from globalState on bindContext', () => {
      const memento = makeMemento({ 'odScanner.seenAgents': ['a', 'b'] });
      bindService(service, memento);
      expect(service.getNewAgents(detected).map((a) => a.id)).toEqual(['c']);
    });
  });

  describe('addCustomAgent() / isCustomAgent()', () => {
    it('isCustomAgent is false before adding', () => {
      expect(service.isCustomAgent('x')).toBe(false);
    });

    it('addCustomAgent marks the agent as custom', () => {
      service.addCustomAgent('my-agent');
      expect(service.isCustomAgent('my-agent')).toBe(true);
    });

    it('addCustomAgent does not duplicate entries', () => {
      const memento = makeMemento();
      bindService(service, memento);
      service.addCustomAgent('my-agent');
      service.addCustomAgent('my-agent');
      expect(memento.store.get('odScanner.customAgents')).toEqual(['my-agent']);
    });

    it('restores custom agents from globalState on bindContext', () => {
      const memento = makeMemento({ 'odScanner.customAgents': ['saved'] });
      bindService(service, memento);
      expect(service.isCustomAgent('saved')).toBe(true);
    });
  });

  describe('onChange()', () => {
    it('notifies listeners on update', () => {
      let calls = 0;
      service.onChange(() => calls++);
      service.update([makeAgent('a')]);
      service.update([]);
      expect(calls).toBe(2);
    });

    it('unsubscribes via the returned function', () => {
      let calls = 0;
      const off = service.onChange(() => calls++);
      off();
      service.update([makeAgent('a')]);
      expect(calls).toBe(0);
    });

    it('a throwing listener does not break other listeners', () => {
      let calls = 0;
      service.onChange(() => {
        throw new Error('boom');
      });
      service.onChange(() => calls++);
      expect(() => service.update([])).not.toThrow();
      expect(calls).toBe(1);
    });
  });
});
