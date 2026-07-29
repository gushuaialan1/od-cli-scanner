# od-cli-scanner Project Status — 2026-07-29

## Current Status
- **Rust core:** v0.1.3, 81 tests 全绿 ✅
- **VS Code extension:** 37 vitest 全绿 ✅
- **CI:** 13/13 jobs 全绿（ubuntu/macos/windows）✅
- **打包:** vsix 156KB 零警告 + 4 平台 release matrix ✅
- **同步:** 本地与 origin/master 完全一致 ✅

## 2026-07-29 完成的工作
1. open-design 测试体系研究报告（.hermes/plans/reference-study-opendesign.md）
2. Rust 集成测试：mock CLI fixtures + EnvGuard + serial 场景（cc0ed70）
3. VS Code vitest 骨架：agentService/terminalSessionManager/types（bce7a57）
4. CI 重构：check/rust-test matrix/fmt/clippy/build-napi/vscode-build（c334398）
5. fmt 历史遗留修复（692cd4a）
6. Windows 测试门控 #[cfg(unix)] + 跨平台声明式断言（bc7c77c）
7. 本地打包验证：156KB 零警告（3d83e2e, 6c0774d）
8. 分平台打包 release matrix（518edd7）

## 发布阻塞（唯一）
需要老顾手动创建 Azure DevOps PAT：
- https://dev.azure.com → User Settings → Personal access tokens
- Org: All accessible organizations
- Scope: Marketplace > Manage
拿到 token 后：验证 publisher → 打 tag v0.1.0 → vsce publish
