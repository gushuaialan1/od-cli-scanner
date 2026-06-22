import {
  scanAgents as nativeScanAgents,
  checkBinary as nativeCheckBinary,
  JsAgentDef,
  JsDetectionResult,
  JsDetectedAgent,
} from './native';

export type {
  JsAgentDef as AgentDef,
  JsDetectionResult as DetectionResult,
  JsDetectedAgent as DetectedAgent,
  JsModelOption as ModelOption,
  JsAuthStatus as AuthStatus,
  JsModelsSource as ModelsSource,
  JsAgentDiagnostic as AgentDiagnostic,
  JsFixAction as FixAction,
} from './native';

// Built-in agent definitions (sync with Rust defaults)
export const DEFAULT_AGENT_DEFS: JsAgentDef[] = [
  {
    id: 'claude',
    name: 'Claude Code',
    bin: 'claude',
    fallback_bins: ['claude-code'],
    version_args: ['--version'],
    version_probe_timeout_ms: 3000,
    fallback_models: [
      { id: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
      { id: 'claude-opus-4', label: 'Claude Opus 4' },
    ],
    stream_format: 'anthropic',
    install_url: 'https://docs.anthropic.com/claude-code',
    docs_url: 'https://docs.anthropic.com/claude-code',
    bin_env_key: 'CLAUDE_BIN',
    auth_probe_args: ['auth', 'status'],
    auth_probe_timeout_ms: 5000,
    list_models_args: ['models'],
    list_models_timeout_ms: 5000,
  },
  // ... more agents
];

export interface ScannerOptions {
  /** Custom agent definitions (replaces defaults if provided) */
  agentDefs?: JsAgentDef[];
  /** Per-agent environment overrides */
  envConfig?: Record<string, Record<string, string>>;
  /** Only return available agents */
  availableOnly?: boolean;
  /** Filter by agent IDs */
  filter?: string[];
}

/**
 * Scan for installed AI coding agents.
 * @returns Promise<DetectionResult>
 */
export async function scanAgents(options: ScannerOptions = {}): Promise<JsDetectionResult> {
  const defs = options.agentDefs ?? DEFAULT_AGENT_DEFS;
  const env = options.envConfig ?? {};

  let result = await nativeScanAgents(defs, env);

  if (options.availableOnly) {
    result = {
      ...result,
      agents: result.agents.filter(a => a.available),
    };
  }

  if (options.filter && options.filter.length > 0) {
    const filterSet = new Set(options.filter);
    result = {
      ...result,
      agents: result.agents.filter(a => filterSet.has(a.id)),
    };
  }

  return result;
}

/**
 * Check if a specific binary is available and get its version.
 * @returns Promise<string | null> version string or null
 */
export async function checkBinary(
  bin: string,
  versionArgs: string[] = ['--version'],
  timeoutMs?: number
): Promise<string | null> {
  return nativeCheckBinary(bin, versionArgs, timeoutMs);
}

// Re-export native functions
export { nativeScanAgents, nativeCheckBinary };
