import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { DetectedAgent, ScannerError, ScannerErrorCode } from './types';

const SCAN_TIMEOUT_MS = 8000; // 8s hard cap (PRD says < 3s ideally)

export class ScannerBridge {
  constructor(private outputChannel: vscode.OutputChannel) {}

  async scan(): Promise<DetectedAgent[]> {
    const binaryPath = this.resolveBinaryPath();
    if (!binaryPath) {
      throw new ScannerError(
        'BINARY_NOT_FOUND',
        'od-scan binary not found. Install od-cli-scanner or set odScanner.binaryPath.'
      );
    }

    return new Promise((resolve, reject) => {
      const proc = spawn(binaryPath, ['--format', 'json'], {
        env: { ...process.env },
        timeout: SCAN_TIMEOUT_MS,
      });

      let stdout = '';
      let stderr = '';
      let killed = false;

      const timer = setTimeout(() => {
        killed = true;
        proc.kill('SIGTERM');
        reject(
          new ScannerError('TIMEOUT', 'od-scan timed out after 8s.')
        );
      }, SCAN_TIMEOUT_MS);

      proc.stdout.on('data', (chunk: Buffer) => {
        stdout += chunk.toString('utf-8');
      });

      proc.stderr.on('data', (chunk: Buffer) => {
        stderr += chunk.toString('utf-8');
      });

      proc.on('error', (err) => {
        clearTimeout(timer);
        reject(
          new ScannerError(
            'BINARY_NOT_FOUND',
            `Failed to spawn od-scan: ${err.message}`,
            err
          )
        );
      });

      proc.on('close', (code) => {
        clearTimeout(timer);
        if (killed) return;

        if (code !== 0) {
          this.outputChannel.appendLine(
            `od-scan exited with code ${code}: ${stderr}`
          );
        }

        try {
          const lines = stdout
            .split('\n')
            .filter((l) => l.trim().startsWith('[') || l.trim().startsWith('{'));
          const jsonText = lines.join('\n');
          const agents: DetectedAgent[] = JSON.parse(jsonText);
          resolve(agents);
        } catch (err) {
          reject(
            new ScannerError(
              'PARSE_ERROR',
              `Failed to parse od-scan output: ${(err as Error).message}`,
              err as Error
            )
          );
        }
      });
    });
  }

  private resolveBinaryPath(): string | undefined {
    const config = vscode.workspace.getConfiguration('odScanner');
    const customPath = config.get<string>('binaryPath');
    if (customPath) {
      return customPath;
    }
    // Auto-discover from PATH
    const candidates = ['od-scan'];
    for (const c of candidates) {
      // Simple heuristic: try to resolve via `which` equivalent
      try {
        const { execSync } = require('child_process');
        const resolved = execSync(`which ${c}`, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'ignore'] });
        if (resolved) {
          return resolved.trim();
        }
      } catch {
        // ignore
      }
    }
    // Fallback: check common local build path
    const localBuild = `${process.env.HOME}/projects/od-cli-scanner/target/debug/od-scan`;
    try {
      const fs = require('fs');
      if (fs.existsSync(localBuild)) {
        return localBuild;
      }
    } catch {
      // ignore
    }
    return undefined;
  }
}
