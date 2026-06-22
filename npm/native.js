const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;
const nativePackage = require('./package.json');
const { name } = nativePackage;

function loadNative() {
  // 1. Try prebuilt binary
  const prebuildPath = join(__dirname, 'prebuilds', `${platform}-${arch}`, `${name}.node`);
  if (existsSync(prebuildPath)) {
    return require(prebuildPath);
  }

  // 2. Try napi-rs build output (development)
  const buildPath = join(__dirname, `${name}.node`);
  if (existsSync(buildPath)) {
    return require(buildPath);
  }

  // 3. Fallback: try to find any .node file
  const fallbackPaths = [
    join(__dirname, `${name}.node`),
    join(__dirname, 'build', 'Release', `${name}.node`),
  ];
  for (const p of fallbackPaths) {
    if (existsSync(p)) {
      return require(p);
    }
  }

  throw new Error(
    `No prebuilt binary found for ${platform}-${arch}. ` +
    `Please build from source: npm run build`
  );
}

module.exports = loadNative();
