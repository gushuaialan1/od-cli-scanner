"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AgentService = void 0;
const RECENT_AGENTS_KEY = 'odScanner.recentAgents';
const MAX_RECENT = 5;
class AgentService {
    agents = [];
    modelsMap = new Map();
    listeners = new Set();
    recentIds = [];
    globalState;
    seenAgents = [];
    customAgents = [];
    SEEN_AGENTS_KEY = 'odScanner.seenAgents';
    CUSTOM_AGENTS_KEY = 'odScanner.customAgents';
    bindContext(context) {
        this.globalState = context.globalState;
        this.recentIds = context.globalState.get(RECENT_AGENTS_KEY, []);
        this.seenAgents = context.globalState.get(this.SEEN_AGENTS_KEY, []);
        this.customAgents = context.globalState.get(this.CUSTOM_AGENTS_KEY, []);
    }
    update(agents) {
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
    getAll() {
        return [...this.agents];
    }
    getAvailable() {
        return this.agents.filter((a) => a.available);
    }
    getById(id) {
        return this.agents.find((a) => a.id === id);
    }
    getModels(agentId) {
        return this.modelsMap.get(agentId);
    }
    getRecentAgents() {
        const available = this.getAvailable();
        const recent = [];
        const others = [];
        for (const agent of available) {
            if (this.recentIds.includes(agent.id)) {
                recent.push(agent);
            }
            else {
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
    recordUsage(agentId) {
        this.recentIds = this.recentIds.filter((id) => id !== agentId);
        this.recentIds.unshift(agentId);
        if (this.recentIds.length > MAX_RECENT) {
            this.recentIds = this.recentIds.slice(0, MAX_RECENT);
        }
        if (this.globalState) {
            this.globalState.update(RECENT_AGENTS_KEY, this.recentIds);
        }
    }
    onChange(listener) {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }
    /**
     * Get agents that were detected but not yet seen/registered by the user.
     */
    getNewAgents(allDetected) {
        return allDetected.filter(a => !this.seenAgents.includes(a.id));
    }
    /**
     * Mark agents as seen. Called when user dismisses notification or adds an agent.
     */
    markAsSeen(agentIds) {
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
    addCustomAgent(agentId) {
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
    isCustomAgent(agentId) {
        return this.customAgents.includes(agentId);
    }
    notify() {
        for (const l of this.listeners) {
            try {
                l();
            }
            catch {
                // ignore listener errors
            }
        }
    }
}
exports.AgentService = AgentService;
//# sourceMappingURL=agentService.js.map