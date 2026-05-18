# GitBox

GitBox 是一个 Tauri + Vue 桌面 Git 客户端，应用标识为 `wang.ncc.gitbox`。首版聚焦 JetBrains/Rebased 风格的本地 Git 工作流：工作区/暂存区、diff、按 hunk 暂存、提交、回滚、shelve/stash，以及通用 Git 远程 fetch/pull/push。

## Features

- 仓库入口：打开本地仓库、clone、init、shallow clone、unshallow。
- 本地变更：工作区/暂存区、按文件和按 hunk 暂存/取消暂存、changelist 分组、回滚、生成/应用 patch。
- 提交：commit、amend、sign-off、GPG signing、author override、提交信息历史、commit and push。
- 日志：all refs 提交列表、提交详情、分支/作者/路径/关键字过滤、按文件应用提交、checkout revision、cherry-pick、revert、reset、undo commit。
- 历史改写：fixup、squash、drop commit、push up to commit、高级 rebase 参数（interactive、onto、autosquash、rebase-merges、keep-empty、root、update-refs）。
- 分支/tag：创建、切换、收藏分支、远程跟踪、upstream、重命名、清理已合并分支、删除本地/远程分支、创建/推送/删除 tag。
- 对比与追踪：ref compare、文件历史、blame。
- 协作结构：worktree 创建/查看/移除、submodule 查看/更新、Git LFS 状态。
- 冲突处理：merge/rebase/cherry-pick/revert 状态识别、continue/abort/skip、三方内容查看、按整文件/冲突块/全部冲突块/手动结果解决。
- 托管与发布：GitHub/GitLab/Bitbucket remote 链接识别，GitHub Actions 多平台 Tauri release workflow。

## Stack

- Tauri 2 + Vue 3 + TypeScript + Vite
- Rust 后端，`git2-rs/libgit2` 作为核心 Git 读写层
- 系统 Git 作为隐藏 fallback，用于 hunk apply、restore、stash、pull 等边界操作
- SQLite 保存最近仓库和 shelve 元数据

## Development

```sh
npm install
npm run tauri dev
```

## Verification

```sh
npm run build
cd src-tauri && cargo test
npm run tauri build
```

macOS 当前通过 `src-tauri/tauri.macos.conf.json` 默认只打 `.app`，避免 DMG 的 Finder AppleScript 在部分环境中阻塞。产物路径：

```text
src-tauri/target/release/bundle/macos/GitBox.app
```
