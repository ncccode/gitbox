# GitBox

GitBox 是一个 Tauri + Vue 桌面 Git 客户端。它面向高频 Git 工作流，目标是把变更管理、提交历史、分支操作和冲突处理放进一个轻量的桌面工作台。

## 开发动机

AI 编程已经进入对话编程模式，开发者越来越少直接翻阅代码细节，而是通过对话、指令和局部上下文推动开发。作者日常需要开发和维护多个项目，也经常与多位同事协作，冲突合并几乎不可避免。

GitBox 因此被设计成一个可以同时管理多项目、内置三栏 Merge 工具的桌面 Git 客户端，用来承接日常开发中的仓库切换、变更查看、提交、同步和冲突处理，让这些高频操作更集中、更顺手。

## 开发说明

本软件完全由 AI 开发，代码、界面和功能实现均通过 AI 辅助完成，因此仍可能存在 bug 或不符合预期的行为。如遇到任何问题，或有功能请求，欢迎提交 issue。

## 主要能力

- 仓库入口：打开本地仓库、批量拖入项目、clone、init、shallow clone 和 unshallow。
- 项目面板：浏览项目文件，读取、保存、新建、复制、移动、重命名和删除文件或目录，并可从应用内打开系统文件管理器或终端。
- 本地变更：查看工作区和暂存区，按文件或 hunk 暂存、取消暂存、丢弃变更，支持 changelist 分组、patch 生成和 patch 应用。
- 提交工作流：commit、amend、sign-off、GPG signing、author override、提交信息历史，以及 commit and push。
- 提交历史：查看 all refs 日志、提交详情和文件 diff，按分支、作者、路径、关键字过滤，支持文件历史、blame 和 ref compare。
- 分支与 tag：创建、切换、收藏、重命名、清理已合并分支，设置 upstream，删除本地/远程分支，创建、推送和删除 tag。
- 历史操作：merge、rebase、advanced rebase、cherry-pick、revert、reset、undo commit、fixup、squash、drop commit 和 push up to commit。
- 远程协作：fetch、pull、smart pull preflight、push、force-with-lease、push tags，以及 remote 新增、修改和删除。
- 暂存与协作结构：shelve/unshelve、stash apply/pop/drop/clear、worktree 创建/查看/移除、submodule 更新和 Git LFS 状态查看。
- 冲突处理：识别 merge/rebase/cherry-pick/revert 状态，支持 continue/abort/skip、三方内容查看、按整文件或冲突块选择 ours/base/theirs，并可手动保存合并结果。

## 技术栈

- 前端：Vue 3、TypeScript、Pinia、Vite、Lucide Vue。
- 桌面壳：Tauri 2，默认窗口尺寸为 `1280x820`，最小尺寸为 `960x640`。
- 后端：Rust，核心 Git 读写基于 `git2-rs/libgit2`。
- Git 边界操作：系统 Git 作为补充执行层，用于 hunk apply、restore、stash、pull、push 等需要完全贴近 CLI 行为的场景。
- 本地数据：前端设置与仓库列表保存在 localStorage，最近仓库和 shelve 元数据保存在 SQLite。

## 项目结构

```text
.
├── src/                    # Vue 应用
│   ├── components/          # 顶栏、项目面板、工作区导航和图标组件
│   ├── composables/         # 应用级交互编排
│   ├── lib/                 # Tauri command 调用封装
│   ├── stores/              # Pinia 状态模块
│   ├── styles/              # 应用样式
│   └── types/               # 前后端共享的 TypeScript 类型
├── src-tauri/               # Tauri 和 Rust 后端
│   ├── src/gitbox.rs        # GitBox 的主要 Git 命令实现
│   ├── tauri.conf.json      # 通用 Tauri 配置
│   └── tauri.macos.conf.json # macOS 本地打包覆盖配置
```

## 本地开发

准备环境：

- Node.js 22 或兼容的现代 Node.js 版本。
- Rust stable toolchain。
- 系统 Git。部分 Git 操作会直接调用本机 `git` 命令。
- Tauri 2 对应平台依赖，Linux 需要 WebKitGTK 等桌面库。

安装依赖并启动桌面开发模式：

```sh
npm install
npm run tauri dev
```

仅启动前端 Vite 服务：

```sh
npm run dev
```

## 验证

前端类型检查和构建：

```sh
npm run build
```

Rust 单元测试：

```sh
cd src-tauri
cargo test
```

完整 Tauri 打包：

```sh
npm run tauri build
```

## 打包与发布

版本号需要保持三处一致：

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

GitHub Actions 会在推送 `v*` tag 或手动触发 `workflow_dispatch` 时构建 macOS、Linux 和 Windows 产物，并创建 draft release。

macOS 本地打包使用 `src-tauri/tauri.macos.conf.json` 将 bundle target 覆盖为 `.app`，避免部分环境中 DMG Finder AppleScript 阻塞。产物路径：

```text
src-tauri/target/release/bundle/macos/GitBox.app
```

## 实现备注

- 冲突编辑器采用三栏视图，纵向滚动联动，横向滚动由各列代码区独立处理，方便在窄窗口下查看长行。
- 提交和 push 前会检查未解决的冲突状态与冲突标记，避免把包含 conflict marker 的内容提交或推送出去。
- `src/lib/gitboxCommands.ts` 是前端访问 Rust command 的集中入口，新增后端能力时应优先在这里补齐类型化封装。
