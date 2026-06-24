"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AgentService = void 0;
class AgentService {
    agents = [];
    listeners = new Set();
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