import { DetectedAgent } from './types';

export class AgentService {
  private agents: DetectedAgent[] = [];
  private listeners: Set<() => void> = new Set();

  update(agents: DetectedAgent[]): void {
    this.agents = agents;
    this.notify();
  }

  getAll(): DetectedAgent[] {
    return [...this.agents];
  }

  getAvailable(): DetectedAgent[] {
    return this.agents.filter((a) => a.available);
  }

  getById(id: string): DetectedAgent | undefined {
    return this.agents.find((a) => a.id === id);
  }

  onChange(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    for (const l of this.listeners) {
      try {
        l();
      } catch {
        // ignore listener errors
      }
    }
  }
}
