// Types matching od-scan JSON output

export interface AgentModel {
  id: string;
  label: string;
}

export interface FixAction {
  kind: string;
  label?: string;
}

export interface Diagnostic {
  kind: string;
  message: string;
  fix_actions?: FixAction[];
}

export interface DetectedAgent {
  id: string;
  name: string;
  bin: string;
  available: boolean;
  path?: string;
  version?: string;
  models?: AgentModel[];
  models_source?: string;
  auth_status?: string;
  diagnostics?: Diagnostic[];
  stream_format?: string;
  install_url?: string;
  docs_url?: string;
  capabilities?: string[];
}

export interface ScanResult {
  agents: DetectedAgent[];
}

export type ScannerErrorCode =
  | 'BINARY_NOT_FOUND'
  | 'TIMEOUT'
  | 'PARSE_ERROR'
  | 'UNKNOWN';

export class ScannerError extends Error {
  constructor(
    public readonly code: ScannerErrorCode,
    message: string,
    public readonly cause?: Error
  ) {
    super(message);
    this.name = 'ScannerError';
  }
}
