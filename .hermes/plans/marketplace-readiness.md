# Marketplace Readiness — od-scanner-vscode 0.1.0

Date: 2026-07-29
Verified locally with `npx vsce package` (@vscode/vsce 2.15.0+).

## 打包产物

- **文件**: `packages/vscode/od-scanner-vscode-0.1.0.vsix`
- **大小**: 156 KB (160,032 bytes)
- **文件数**: 50
- **vsce 输出**: 0 ERROR, 0 WARNING

## 产物内容核查

包含（必需）：
- `extension/out/*.js` — 编译产物（11 个模块，入口 `out/extension.js`）
- `extension/media/xterm.css`
- `extension/package.json`、`extension/readme.md`、`extension/LICENSE.txt`
- `extension/node_modules/@xterm/xterm/lib/xterm.js`（webview 终端渲染）
- `extension/node_modules/@xterm/addon-fit/lib/addon-fit.js`
- `extension/node_modules/node-pty/lib/*` + `build/Release/pty.node`（运行时原生依赖）

不包含（已排除）：
- `src/`、`tsconfig.json`、`vitest.config.ts`、`.hermes/`、`.vscode/`、`.git/`
- `out/test/`、`**/*.map`（sourcemaps）
- `node_modules/nan/`（node-pty 构建期头文件依赖，运行时不需要）
- `node_modules/node-pty/{deps,src,scripts,typings}` 及 build 中间产物（Makefile/config.gypi/*.mk）
- `node_modules/@xterm/*/{src,typings}`（webview 只用 lib/ 打包产物）

## 遇到的问题与处理

| 级别 | 问题 | 处理 |
|------|------|------|
| ERROR | `Extension entrypoint(s) missing: extension/out/extension.js` | 旧 `.vscodeignore` 误排除 `out/` 和整个 `node_modules/`。重写 `.vscodeignore`：保留 `out/` 与运行时依赖（@xterm、node-pty），仅排除测试/构建/源码类内容 |
| 隐患 | 扩展目录内无 LICENSE 文件 | 从仓库根目录复制 `LICENSE` 到 `packages/vscode/LICENSE`（vsce 打包为 `LICENSE.txt`） |
| 体积 | node-pty 的 `deps/`(1.3M)、`src/`、build 中间产物；nan(444K)；xterm 源码 | `.vscodeignore` 精确排除，vsix 从 ~530KB 降到 156KB（50 文件） |
| 清理 | 旧产物 `od-scanner-vscode-0.1.0.vsix` | 已删除并重新打包 |

## 无法本地解决 / 待发布前处理

1. **扩展图标**：`package.json` 未声明 `icon`。Marketplace 列表会显示默认图标，建议补一个 128×128 PNG（如 `media/icon.png`）并在 package.json 加 `"icon": "media/icon.png"`。
2. **galleryBanner**：未设置，可选（影响 Marketplace 页面横幅配色）。
3. **原生模块平台限制**：vsix 内的 `node-pty/build/Release/pty.node` 是**本机 Linux x64 编译**的二进制。
   - 发布跨平台需用 `vsce package --target <platform>`（如 `linux-x64`, `win32-x64`, `darwin-x64`, `darwin-arm64`）分别打包发布，或切换到带 prebuilds 的 node-pty 分发方式。
   - 当前 vsix 只保证在 Linux x64 上可用。
4. **publisher 验证**：publisher `gushuaialan1` 需在 Marketplace 完成验证（尤其 README 中如有外链需与验证域名一致）。

## 发布检查清单

- [x] `publisher` 字段设置（gushuaialan1）
- [x] `repository` 字段设置
- [x] README.md 存在
- [x] LICENSE 包含在 vsix 内
- [x] `keywords`、`categories` 已配置
- [x] `.vscodeignore` 精简产物（无源码/测试/sourcemap）
- [ ] 扩展图标（128×128 PNG）
- [ ] publisher 在 Marketplace 验证通过
- [ ] 创建 PAT（Azure DevOps Personal Access Token）
  - Scope 要求：**Marketplace → Acquire & Manage**（通常直接给 `Marketplace (Manage)`）
  - Organization 需选 **All accessible organizations**
- [ ] 决定发布策略：单平台 vsix 或 `--target` 多平台包
- [ ] `vsce publish`（或 `vsce publish --packagePath *.vsix`）

## 复现命令

```bash
cd packages/vscode
npm ci && npm run compile
npx vsce package        # 产出 od-scanner-vscode-0.1.0.vsix
unzip -l od-scanner-vscode-0.1.0.vsix | wc -l   # 文件清单
```
