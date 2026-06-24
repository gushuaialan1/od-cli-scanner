"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScannerBridge = void 0;
const vscode = __importStar(require("vscode"));
const child_process_1 = require("child_process");
const types_1 = require("./types");
const SCAN_TIMEOUT_MS = 8000; // 8s hard cap (PRD says < 3s ideally)
class ScannerBridge {
    outputChannel;
    constructor(outputChannel) {
        this.outputChannel = outputChannel;
    }
    async scan(cwd) {
        const binaryPath = this.resolveBinaryPath();
        if (!binaryPath) {
            throw new types_1.ScannerError('BINARY_NOT_FOUND', 'od-scan binary not found. Install od-cli-scanner or set odScanner.binaryPath.');
        }
        return new Promise((resolve, reject) => {
            const proc = (0, child_process_1.spawn)(binaryPath, ['--format', 'json'], {
                env: { ...process.env },
                timeout: SCAN_TIMEOUT_MS,
                cwd,
            });
            let stdout = '';
            let stderr = '';
            let killed = false;
            const timer = setTimeout(() => {
                killed = true;
                proc.kill('SIGTERM');
                reject(new types_1.ScannerError('TIMEOUT', 'od-scan timed out after 8s.'));
            }, SCAN_TIMEOUT_MS);
            proc.stdout.on('data', (chunk) => {
                stdout += chunk.toString('utf-8');
            });
            proc.stderr.on('data', (chunk) => {
                stderr += chunk.toString('utf-8');
            });
            proc.on('error', (err) => {
                clearTimeout(timer);
                reject(new types_1.ScannerError('BINARY_NOT_FOUND', `Failed to spawn od-scan: ${err.message}`, err));
            });
            proc.on('close', (code) => {
                clearTimeout(timer);
                if (killed)
                    return;
                if (code !== 0) {
                    this.outputChannel.appendLine(`od-scan exited with code ${code}: ${stderr}`);
                }
                try {
                    const lines = stdout
                        .split('\n')
                        .filter((l) => l.trim().startsWith('[') || l.trim().startsWith('{'));
                    const jsonText = lines.join('\n');
                    const agents = JSON.parse(jsonText);
                    resolve(agents);
                }
                catch (err) {
                    reject(new types_1.ScannerError('PARSE_ERROR', `Failed to parse od-scan output: ${err.message}`, err));
                }
            });
        });
    }
    resolveBinaryPath() {
        const config = vscode.workspace.getConfiguration('odScanner');
        const customPath = config.get('binaryPath');
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
            }
            catch {
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
        }
        catch {
            // ignore
        }
        return undefined;
    }
}
exports.ScannerBridge = ScannerBridge;
//# sourceMappingURL=scannerBridge.js.map