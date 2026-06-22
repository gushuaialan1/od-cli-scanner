const { existsSync } = require('fs');
const { join } = require('path');

// Try to load the native binding for current platform
const bindings = [
  'od-cli-scanner.linux-x64-gnu.node',
  'od-cli-scanner.darwin-x64.node',
  'od-cli-scanner.darwin-arm64.node',
  'od-cli-scanner.win32-x64-msvc.node',
];

let native = null;
for (const binding of bindings) {
  const path = join(__dirname, binding);
  if (existsSync(path)) {
    native = require(path);
    break;
  }
}

if (!native) {
  throw new Error(
    'od-cli-scanner native binding not found for current platform. ' +
    'Please install the correct prebuild or build from source.'
  );
}

module.exports = {
  scanAgents: native.scanAgents,
  checkBinary: native.checkBinary,
};
