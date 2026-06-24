"use strict";
// Types matching od-scan JSON output
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScannerError = void 0;
class ScannerError extends Error {
    code;
    cause;
    constructor(code, message, cause) {
        super(message);
        this.code = code;
        this.cause = cause;
        this.name = 'ScannerError';
    }
}
exports.ScannerError = ScannerError;
//# sourceMappingURL=types.js.map