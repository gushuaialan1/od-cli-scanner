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
  private seenAgents: string[] = [];
  private customAgents: string[] = [];
  private readonly SEEN_AGENTS_KEY = 'odScanner.seenAgents';
  private readonly CUSTOM_AGENTS_KEY = 'odScanner.customAgents';

  bindContext(context: vscode.ExtensionContext): void {
    this.globalState = context.globalState;
    this.recentIds = context.globalState.get<string[]>(RECENT_AGENTS_KEY, []);
    this.seenAgents = context.globalState.get<string[]>(this.SEEN_AGENTS_KEY, []);
    this.customAgents = context.globalState.get<string[]>(this.CUSTOM_AGENTS_KEY, []);
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

  /**
   * Get agents that were detected but not yet seen/registered by the user.
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
