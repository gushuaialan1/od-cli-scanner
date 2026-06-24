import * as vscode from 'vscode';
import { DetectedAgent, AgentModel } from './types';

const RECENT_AGENTS_KEY = 'odScanner.recentAgents';
const MAX_RECENT = 5;

export class AgentService {
  private agents: DetectedAgent[] = [];
  private modelsMap: Map<string, AgentModel[]> = new Map();
  private listeners: Set<() => void> = new Set();
  private recentIds: string[] = [];
  private globalState: vscode.Memento | undefined;

  bindContext(context: vscode.ExtensionContext): void {
    this.globalState = context.globalState;
    this.recentIds = context.globalState.get<string[]>(RECENT_AGENTS_KEY, []);
  }

  update(agents: DetectedAgent[]): void {
    this.agents = agents;
    // Build models map from agent data
    this.modelsMap.clear();
    for (const agent of agents) {
      if (agent.models && agent.models.length > 0) {
        this.modelsMap.set(agent.id, agent.models);
      }
    }
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

  getModels(agentId: string): AgentModel[] | undefined {
    return this.modelsMap.get(agentId);
  }

  getRecentAgents(): DetectedAgent[] {
    const available = this.getAvailable();
    const recent: DetectedAgent[] = [];
    const others: DetectedAgent[] = [];
    for (const agent of available) {
      if (this.recentIds.includes(agent.id)) {
        recent.push(agent);
      } else {
        others.push(agent);
      }
    }
    recent.sort((a, b) => {
      const idxA = this.recentIds.indexOf(a.id);
      const idxB = this.recentIds.indexOf(b.id);
      return idxA - idxB;
    });
    return [...recent, ...others];
  }

  recordUsage(agentId: string): void {
    this.recentIds = this.recentIds.filter((id) => id !== agentId);
    this.recentIds.unshift(agentId);
    if (this.recentIds.length > MAX_RECENT) {
      this.recentIds = this.recentIds.slice(0, MAX_RECENT);
    }
    if (this.globalState) {
      this.globalState.update(RECENT_AGENTS_KEY, this.recentIds);
    }
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
