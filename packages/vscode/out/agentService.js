"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AgentService = void 0;
const RECENT_AGENTS_KEY = 'odScanner.recentAgents';
const MAX_RECENT = 5;
class AgentService {
    agents = [];
    listeners = new Set();
    recentIds = [];
    globalState;
    bindContext(context) {
        this.globalState = context.globalState;
        this.recentIds = context.globalState.get(RECENT_AGENTS_KEY, []);
    }
    update(agents) {
        this.agents = agents;
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