use git2::{BranchType, DiffFormat, DiffOptions, Oid, Repository, Status, StatusOptions};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

#[derive(Debug, thiserror::Error)]
pub enum GitboxError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl GitboxError {
    pub fn command(self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    pub name: String,
    pub url: Option<String>,
    pub push_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInfo {
    pub path: String,
    pub workdir: Option<String>,
    pub git_dir: String,
    pub is_bare: bool,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub remotes: Vec<RemoteInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchSummary {
    pub current_branch: Option<String>,
    pub upstream: Option<String>,
    pub head: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub detached: bool,
    pub clean: bool,
    pub remotes: Vec<RemoteInfo>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
    pub ignored: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: String,
    pub old_path: Option<String>,
    pub kind: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub ignored: bool,
    pub conflicted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub repo: RepositoryInfo,
    pub branch: BranchSummary,
    pub files: Vec<ChangedFile>,
    pub counts: StatusCounts,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub index: usize,
    pub header: String,
    pub old_start: i32,
    pub old_lines: i32,
    pub new_start: i32,
    pub new_lines: i32,
    pub patch: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResponse {
    pub path: Option<String>,
    pub staged: bool,
    pub text: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub ok: bool,
    pub message: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResult {
    pub oid: String,
    pub summary: BranchSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfInfo {
    pub id: Option<i64>,
    pub repo_path: String,
    pub message: String,
    pub stash_ref: String,
    pub stash_oid: String,
    pub created_at: i64,
    pub applied_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ShelfDraft {
    pub message: String,
    pub stash_ref: String,
    pub stash_oid: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitSummary {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    pub body: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub committer_time: i64,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitFileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetails {
    pub commit: CommitSummary,
    pub files: Vec<CommitFileChange>,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    pub name: String,
    pub full_name: String,
    pub branch_type: String,
    pub current: bool,
    pub upstream: Option<String>,
    pub target: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagInfo {
    pub name: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryEntry {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameLine {
    pub line_number: usize,
    pub oid: String,
    pub short_oid: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefComparisonCommit {
    pub side: String,
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefComparison {
    pub left: String,
    pub right: String,
    pub commits: Vec<RefComparisonCommit>,
    pub files: Vec<CommitFileChange>,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
    pub prunable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StashInfo {
    pub stash_ref: String,
    pub oid: String,
    pub message: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmoduleInfo {
    pub path: String,
    pub oid: String,
    pub status: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchList {
    pub current: Option<String>,
    pub branches: Vec<BranchInfo>,
    pub tags: Vec<TagInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOperationState {
    pub active: bool,
    pub operation: Option<String>,
    pub conflicted_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictBlock {
    pub index: usize,
    pub ours: String,
    pub base: Option<String>,
    pub theirs: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetails {
    pub path: String,
    pub base: Option<String>,
    pub ours: Option<String>,
    pub theirs: Option<String>,
    pub current: Option<String>,
    pub blocks: Vec<ConflictBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileEntry {
    pub path: String,
    pub name: String,
    pub parent: Option<String>,
    pub depth: usize,
    pub directory: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileMutation {
    pub path: String,
    pub directory: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileContent {
    pub path: String,
    pub content: Option<String>,
    pub binary: bool,
    pub size: u64,
}

#[derive(Debug, Clone)]
struct ParsedConflictBlock {
    index: usize,
    start: usize,
    end: usize,
    ours: Vec<String>,
    base: Vec<String>,
    theirs: Vec<String>,
}

struct GitProcessOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

impl GitProcessOutput {
    fn combined_output(&self) -> String {
        match (self.stdout.trim().is_empty(), self.stderr.trim().is_empty()) {
            (true, true) => String::new(),
            (false, true) => self.stdout.clone(),
            (true, false) => self.stderr.clone(),
            (false, false) => format!("{}{}", self.stdout, self.stderr),
        }
    }

    fn failure_message(&self) -> String {
        let combined = self.combined_output();
        if combined.trim().is_empty() {
            "git 命令执行失败".to_string()
        } else {
            combined
        }
    }
}

pub fn open_repo_core(path: &str) -> Result<RepositoryInfo, GitboxError> {
    let repo = Repository::discover(path)?;
    repository_info(&repo)
}

pub fn list_project_files_core(
    path: &str,
    limit: Option<usize>,
) -> Result<Vec<ProjectFileEntry>, GitboxError> {
    let repo = Repository::discover(path)?;
    let root = fs::canonicalize(repo_workdir(&repo)?)?;
    let max_entries = limit.unwrap_or(1400).clamp(1, 5000);
    let mut entries = Vec::new();
    collect_project_entries(&root, &root, 0, max_entries, &mut entries)?;
    Ok(entries)
}

pub fn read_project_file_core(
    path: &str,
    file_path: String,
) -> Result<ProjectFileContent, GitboxError> {
    let repo = Repository::discover(path)?;
    let root = fs::canonicalize(repo_workdir(&repo)?)?;
    let relative = clean_project_relative_path(file_path)?;
    let target = fs::canonicalize(root.join(&relative))?;

    if !target.starts_with(&root) {
        return Err(GitboxError::Message("文件不在当前项目内".to_string()));
    }

    let metadata = fs::metadata(&target)?;
    if metadata.is_dir() {
        return Err(GitboxError::Message("请选择文件而不是目录".to_string()));
    }
    if metadata.len() > 1_000_000 {
        return Err(GitboxError::Message(
            "文件超过 1MB，暂不在项目视图中预览".to_string(),
        ));
    }

    let bytes = fs::read(&target)?;
    let content = std::str::from_utf8(&bytes)
        .map(|value| value.to_string())
        .ok();
    Ok(ProjectFileContent {
        path: repo_path_string(&relative),
        binary: content.is_none(),
        content,
        size: metadata.len(),
    })
}

pub fn save_project_file_core(
    path: &str,
    file_path: String,
    content: String,
) -> Result<ProjectFileContent, GitboxError> {
    let repo = Repository::discover(path)?;
    let root = fs::canonicalize(repo_workdir(&repo)?)?;
    let relative = clean_project_relative_path(file_path)?;
    let target = fs::canonicalize(root.join(&relative))?;

    if !target.starts_with(&root) {
        return Err(GitboxError::Message("文件不在当前项目内".to_string()));
    }
    if fs::metadata(&target)?.is_dir() {
        return Err(GitboxError::Message("请选择文件而不是目录".to_string()));
    }

    fs::write(&target, content.as_bytes())?;
    let metadata = fs::metadata(&target)?;
    Ok(ProjectFileContent {
        path: repo_path_string(&relative),
        binary: false,
        content: Some(content),
        size: metadata.len(),
    })
}

pub fn create_project_file_core(
    path: &str,
    directory_path: Option<String>,
    name: String,
) -> Result<ProjectFileMutation, GitboxError> {
    let root = project_workdir_root(path)?;
    let parent = resolve_project_directory(&root, directory_path)?;
    let relative = clean_project_child_path(name, "请输入文件名")?;
    let target = parent.join(&relative);
    if target.exists() {
        return Err(GitboxError::Message("同名文件或文件夹已存在".to_string()));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::File::create(&target)?;
    project_file_mutation(
        &root,
        &target,
        false,
        format!("已新建文件 {}", repo_path_string(&relative)),
    )
}

pub fn create_project_directory_core(
    path: &str,
    directory_path: Option<String>,
    name: String,
) -> Result<ProjectFileMutation, GitboxError> {
    let root = project_workdir_root(path)?;
    let parent = resolve_project_directory(&root, directory_path)?;
    let relative = clean_project_child_path(name, "请输入文件夹名")?;
    let target = parent.join(&relative);
    if target.exists() {
        return Err(GitboxError::Message("同名文件或文件夹已存在".to_string()));
    }

    fs::create_dir_all(&target)?;
    project_file_mutation(
        &root,
        &target,
        true,
        format!("已新建文件夹 {}", repo_path_string(&relative)),
    )
}

pub fn rename_project_entry_core(
    path: &str,
    file_path: String,
    new_name: String,
) -> Result<ProjectFileMutation, GitboxError> {
    let root = project_workdir_root(path)?;
    let source = resolve_project_entry(&root, file_path)?;
    let metadata = fs::metadata(&source)?;
    let name = clean_project_entry_name(new_name)?;
    let current_name = source
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| GitboxError::Message("无法重命名项目根目录".to_string()))?;
    if name == current_name {
        return project_file_mutation(&root, &source, metadata.is_dir(), "名称未变化".to_string());
    }

    let parent = source
        .parent()
        .ok_or_else(|| GitboxError::Message("无法读取项目文件路径".to_string()))?;
    let target = parent.join(&name);
    if target.exists() {
        return Err(GitboxError::Message("同名文件或文件夹已存在".to_string()));
    }

    fs::rename(&source, &target)?;
    project_file_mutation(
        &root,
        &target,
        metadata.is_dir(),
        format!("已重命名为 {name}"),
    )
}

pub fn delete_project_entry_core(
    path: &str,
    file_path: String,
) -> Result<CommandResult, GitboxError> {
    let root = project_workdir_root(path)?;
    let source = resolve_project_entry(&root, file_path)?;
    let name = source
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "项目文件".to_string());
    let metadata = fs::metadata(&source)?;

    if metadata.is_dir() {
        fs::remove_dir_all(&source)?;
    } else {
        fs::remove_file(&source)?;
    }

    Ok(CommandResult {
        ok: true,
        message: format!("已删除 {name}"),
        output: String::new(),
    })
}

pub fn copy_project_entry_core(
    path: &str,
    source_path: String,
    target_directory_path: Option<String>,
) -> Result<ProjectFileMutation, GitboxError> {
    let root = project_workdir_root(path)?;
    let source = resolve_project_entry(&root, source_path)?;
    let target_parent = resolve_project_directory(&root, target_directory_path)?;
    let metadata = fs::metadata(&source)?;

    if metadata.is_dir() && target_parent.starts_with(&source) {
        return Err(GitboxError::Message(
            "不能将文件夹复制到自身或子文件夹内".to_string(),
        ));
    }

    let name = source
        .file_name()
        .ok_or_else(|| GitboxError::Message("无法复制项目根目录".to_string()))?;
    let target = unique_project_copy_path(&target_parent.join(name), metadata.is_dir());
    copy_project_entry_recursive(&source, &target)?;
    project_file_mutation(
        &root,
        &target,
        metadata.is_dir(),
        format!("已复制 {}", name.to_string_lossy()),
    )
}

pub fn move_project_entry_core(
    path: &str,
    source_path: String,
    target_directory_path: Option<String>,
) -> Result<ProjectFileMutation, GitboxError> {
    let root = project_workdir_root(path)?;
    let source = resolve_project_entry(&root, source_path)?;
    let target_parent = resolve_project_directory(&root, target_directory_path)?;
    let metadata = fs::metadata(&source)?;

    if metadata.is_dir() && target_parent.starts_with(&source) {
        return Err(GitboxError::Message(
            "不能将文件夹移动到自身或子文件夹内".to_string(),
        ));
    }

    let name = source
        .file_name()
        .ok_or_else(|| GitboxError::Message("无法移动项目根目录".to_string()))?;
    let target = target_parent.join(name);
    if target == source {
        return project_file_mutation(&root, &source, metadata.is_dir(), "位置未变化".to_string());
    }
    if target.exists() {
        return Err(GitboxError::Message("目标位置已有同名项目".to_string()));
    }

    fs::rename(&source, &target)?;
    project_file_mutation(
        &root,
        &target,
        metadata.is_dir(),
        format!("已移动 {}", name.to_string_lossy()),
    )
}

pub fn init_repository_core(
    path: String,
    bare: bool,
    initial_branch: Option<String>,
) -> Result<RepositoryInfo, GitboxError> {
    let repo_path = PathBuf::from(clean_ref_input(path, "请输入仓库目录")?);
    fs::create_dir_all(&repo_path)?;
    let mut args = vec!["init".to_string()];
    if bare {
        args.push("--bare".to_string());
    }
    if let Some(branch) = initial_branch.filter(|value| !value.trim().is_empty()) {
        args.push("--initial-branch".to_string());
        args.push(branch);
    }
    run_git(&repo_path, args, None)?;
    open_repo_core(&path_string(&repo_path))
}

pub fn clone_repository_core(
    url: String,
    directory: String,
    shallow_depth: Option<u32>,
) -> Result<RepositoryInfo, GitboxError> {
    let url = clean_ref_input(url, "请输入仓库地址")?;
    let target = PathBuf::from(clean_ref_input(directory, "请输入保存目录")?);
    let parent = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| GitboxError::Message("保存目录无效".to_string()))?;
    fs::create_dir_all(parent)?;
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| GitboxError::Message("保存目录无效".to_string()))?
        .to_string();

    let mut args = vec!["clone".to_string()];
    if let Some(depth) = shallow_depth.filter(|depth| *depth > 0) {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }
    args.push(url);
    args.push(name);
    run_git(parent, args, None)?;
    open_repo_core(&path_string(&target))
}

pub fn unshallow_repository_core(
    path: &str,
    remote_name: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["fetch".to_string(), "--unshallow".to_string()];
    if let Some(remote) = remote_name.filter(|value| !value.trim().is_empty()) {
        args.push(remote);
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: "已补全浅克隆仓库历史".to_string(),
        output,
    })
}

pub fn repo_status_core(path: &str, include_ignored: bool) -> Result<RepoStatus, GitboxError> {
    let repo = Repository::discover(path)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .include_ignored(include_ignored);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut files = Vec::new();
    let mut counts = StatusCounts::default();

    for entry in statuses.iter() {
        let status = entry.status();
        let staged = has_index_change(status);
        let unstaged = has_worktree_change(status);
        let untracked = status.contains(Status::WT_NEW);
        let ignored = status.contains(Status::IGNORED);
        let conflicted = status.contains(Status::CONFLICTED);

        if staged {
            counts.staged += 1;
        }
        if unstaged {
            counts.unstaged += 1;
        }
        if untracked {
            counts.untracked += 1;
        }
        if ignored {
            counts.ignored += 1;
        }
        if conflicted {
            counts.conflicted += 1;
        }

        let path = entry
            .path()
            .map(ToOwned::to_owned)
            .or_else(|| entry.index_to_workdir().and_then(delta_new_path))
            .or_else(|| entry.head_to_index().and_then(delta_new_path))
            .unwrap_or_else(|| "<unknown>".to_string());

        let old_path = entry
            .head_to_index()
            .and_then(delta_old_path)
            .or_else(|| entry.index_to_workdir().and_then(delta_old_path))
            .filter(|old| old != &path);

        files.push(ChangedFile {
            path,
            old_path,
            kind: status_kind(status),
            staged,
            unstaged,
            untracked,
            ignored,
            conflicted,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(RepoStatus {
        repo: repository_info(&repo)?,
        branch: branch_summary_core(path, include_ignored)?,
        files,
        counts,
    })
}

pub fn branch_summary_core(
    path: &str,
    include_ignored: bool,
) -> Result<BranchSummary, GitboxError> {
    let repo = Repository::discover(path)?;
    let head = repo.head().ok();
    let current_branch = current_branch_name(&repo);
    let head_oid = head
        .as_ref()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string());
    let detached = head.as_ref().map(|head| !head.is_branch()).unwrap_or(false);

    let mut upstream = None;
    let mut ahead = 0;
    let mut behind = 0;

    if let Some(branch_name) = current_branch.as_deref() {
        if let Ok(branch) = repo.find_branch(branch_name, BranchType::Local) {
            if let Ok(upstream_branch) = branch.upstream() {
                upstream = upstream_branch.name().ok().flatten().map(ToOwned::to_owned);
                if let (Some(local_oid), Some(upstream_oid)) =
                    (branch.get().target(), upstream_branch.get().target())
                {
                    if let Ok((a, b)) = repo.graph_ahead_behind(local_oid, upstream_oid) {
                        ahead = a;
                        behind = b;
                    }
                }
            }
        }
    }

    Ok(BranchSummary {
        current_branch,
        upstream,
        head: head_oid,
        ahead,
        behind,
        detached,
        clean: repo_is_clean(&repo, include_ignored)?,
        remotes: remote_infos(&repo)?,
    })
}

pub fn get_diff_core(
    path: &str,
    file_path: Option<String>,
    staged: bool,
) -> Result<DiffResponse, GitboxError> {
    let repo = Repository::discover(path)?;
    let mut opts = DiffOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_typechange(true);

    let normalized_path = match file_path {
        Some(path) if !path.trim().is_empty() => {
            let rel = normalize_repo_path(&repo, &path)?;
            let pathspec = repo_path_string(&rel);
            opts.pathspec(&pathspec);
            Some(pathspec)
        }
        _ => None,
    };

    let diff = if staged {
        let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
        repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))?
    };

    let mut text = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        if matches!(line.origin(), '+' | '-' | ' ') {
            text.push(line.origin());
        }
        text.push_str(&String::from_utf8_lossy(line.content()));
        true
    })?;

    Ok(DiffResponse {
        path: normalized_path,
        staged,
        hunks: parse_diff_hunks(&text),
        text,
    })
}

pub fn stage_paths_core(path: &str, paths: Vec<String>) -> Result<CommandResult, GitboxError> {
    if paths.is_empty() {
        return Err(GitboxError::Message("请选择要暂存的文件".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut index = repo.index()?;

    for path in &paths {
        let rel = normalize_repo_path(&repo, path)?;
        let full_path = workdir.join(&rel);
        if full_path.exists() {
            if let Err(err) = index.add_path(&rel) {
                let args = git_args_with_paths(&["add"], &[repo_path_string(&rel)]);
                let output = run_git(&workdir, args, None).map_err(|fallback| {
                    GitboxError::Message(format!(
                        "libgit2 暂存失败：{err}; 备用 git 命令也失败：{fallback}"
                    ))
                })?;
                return Ok(CommandResult {
                    ok: true,
                    message: "已通过备用 Git 命令暂存选中文件".to_string(),
                    output,
                });
            }
        } else if let Err(err) = index.remove_path(&rel) {
            let args = git_args_with_paths(&["add"], &[repo_path_string(&rel)]);
            let output = run_git(&workdir, args, None).map_err(|fallback| {
                GitboxError::Message(format!(
                    "libgit2 记录删除失败：{err}; 备用 git 命令也失败：{fallback}"
                ))
            })?;
            return Ok(CommandResult {
                ok: true,
                message: "已通过备用 Git 命令暂存删除记录".to_string(),
                output,
            });
        }
    }

    index.write()?;
    Ok(CommandResult {
        ok: true,
        message: "已暂存选中文件".to_string(),
        output: String::new(),
    })
}

pub fn unstage_paths_core(path: &str, paths: Vec<String>) -> Result<CommandResult, GitboxError> {
    if paths.is_empty() {
        return Err(GitboxError::Message("请选择要取消暂存的文件".to_string()));
    }
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let args = git_args_with_paths(&["reset"], &paths);
    run_git(&workdir, args, None).map(|output| CommandResult {
        ok: true,
        message: "已取消暂存".to_string(),
        output,
    })
}

pub fn stage_hunks_core(
    path: &str,
    patches: Vec<String>,
    mode: String,
) -> Result<CommandResult, GitboxError> {
    if patches.is_empty() {
        return Err(GitboxError::Message("请选择要操作的差异块".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let args: Vec<String> = match mode.as_str() {
        "stage" => ["apply", "--cached", "--whitespace=nowarn", "-"]
            .iter()
            .map(|value| value.to_string())
            .collect(),
        "unstage" => ["apply", "--cached", "--reverse", "--whitespace=nowarn", "-"]
            .iter()
            .map(|value| value.to_string())
            .collect(),
        "discard" => ["apply", "--reverse", "--whitespace=nowarn", "-"]
            .iter()
            .map(|value| value.to_string())
            .collect(),
        _ => {
            return Err(GitboxError::Message(
                "差异块操作模式只支持 stage、unstage 或 discard".to_string(),
            ))
        }
    };

    let mut output = String::new();
    for patch in patches {
        output.push_str(&run_git(&workdir, args.clone(), Some(&patch))?);
    }

    Ok(CommandResult {
        ok: true,
        message: if mode == "stage" {
            "已暂存选中块".to_string()
        } else if mode == "discard" {
            "已撤回选中块".to_string()
        } else {
            "已取消暂存选中块".to_string()
        },
        output,
    })
}

pub fn discard_changes_core(path: &str, paths: Vec<String>) -> Result<CommandResult, GitboxError> {
    if paths.is_empty() {
        return Err(GitboxError::Message("请选择要回滚的文件".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    for path in &paths {
        let status = repo.status_file(Path::new(path)).unwrap_or(Status::WT_NEW);
        if status.contains(Status::WT_NEW) && !has_index_change(status) {
            run_git(
                &workdir,
                git_args_with_paths(&["clean", "-fd"], &[path.to_string()]),
                None,
            )?;
        } else {
            run_git(
                &workdir,
                git_args_with_paths(&["restore", "--staged", "--worktree"], &[path.to_string()]),
                None,
            )?;
        }
    }

    Ok(CommandResult {
        ok: true,
        message: "已回滚选中变更".to_string(),
        output: String::new(),
    })
}

pub fn shelve_changes_core(
    path: &str,
    paths: Vec<String>,
    message: Option<String>,
) -> Result<ShelfDraft, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let label = message
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "未命名搁置".to_string());
    let stash_message = format!("gitbox: {label}");
    let mut args = vec![
        "stash".to_string(),
        "push".to_string(),
        "-u".to_string(),
        "-m".to_string(),
        stash_message,
    ];
    if !paths.is_empty() {
        args.push("--".to_string());
        args.extend(paths);
    }

    let output = run_git(&workdir, args, None)?;
    if output.contains("No local changes") || output.contains("没有本地修改") {
        return Err(GitboxError::Message("没有可搁置的本地变更".to_string()));
    }

    let list = run_git(
        &workdir,
        vec![
            "stash".to_string(),
            "list".to_string(),
            "--format=%H%x09%gd%x09%s".to_string(),
        ],
        None,
    )?;
    let first = list
        .lines()
        .next()
        .ok_or_else(|| GitboxError::Message("搁置成功但没有找到贮藏记录".to_string()))?;
    let mut parts = first.splitn(3, '\t');
    let stash_oid = parts.next().unwrap_or_default().to_string();
    let stash_ref = parts.next().unwrap_or("stash@{0}").to_string();

    Ok(ShelfDraft {
        message: label,
        stash_ref,
        stash_oid,
        created_at: now_unix(),
    })
}

pub fn unshelve_core(path: &str, stash_ref: String) -> Result<CommandResult, GitboxError> {
    if stash_ref.trim().is_empty() {
        return Err(GitboxError::Message("请选择要恢复的搁置记录".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec!["stash".to_string(), "apply".to_string(), stash_ref],
        None,
    )?;

    Ok(CommandResult {
        ok: true,
        message: "已恢复搁置内容".to_string(),
        output,
    })
}

pub fn delete_shelf_core(
    app: &AppHandle,
    repo_path: &str,
    stash_ref: String,
    drop_stash: bool,
) -> Result<CommandResult, GitboxError> {
    let stash_ref = clean_ref_input(stash_ref, "请选择要删除的搁置记录")?;
    let repo = Repository::discover(repo_path)?;
    let workdir = repo_workdir(&repo)?;
    let mut output = String::new();

    if drop_stash {
        output = run_git(
            &workdir,
            vec!["stash".to_string(), "drop".to_string(), stash_ref.clone()],
            None,
        )?;
    }

    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    conn.execute(
        "DELETE FROM shelves WHERE repo_path = ?1 AND stash_ref = ?2",
        params![repo_path, stash_ref],
    )?;

    Ok(CommandResult {
        ok: true,
        message: "已删除搁置记录".to_string(),
        output,
    })
}

#[allow(dead_code)]
pub fn commit_core(path: &str, message: String) -> Result<CommitResult, GitboxError> {
    commit_with_options_core(path, message, false, false)
}

pub fn commit_with_options_core(
    path: &str,
    message: String,
    amend: bool,
    sign_off: bool,
) -> Result<CommitResult, GitboxError> {
    commit_with_full_options_core(path, message, amend, sign_off, false, None)
}

pub fn commit_with_full_options_core(
    path: &str,
    message: String,
    amend: bool,
    sign_off: bool,
    gpg_sign: bool,
    author: Option<String>,
) -> Result<CommitResult, GitboxError> {
    if message.trim().is_empty() {
        return Err(GitboxError::Message("提交信息不能为空".to_string()));
    }

    let repo = Repository::discover(path)?;
    let staged_diff = get_diff_core(path, None, true)?;
    if !amend && staged_diff.text.trim().is_empty() {
        return Err(GitboxError::Message("没有已暂存的变更可提交".to_string()));
    }

    let author = clean_optional_arg(author);
    if gpg_sign || author.is_some() {
        let workdir = repo_workdir(&repo)?;
        let mut args = vec!["commit".to_string()];
        if amend {
            args.push("--amend".to_string());
        }
        if sign_off {
            args.push("--signoff".to_string());
        }
        if gpg_sign {
            args.push("-S".to_string());
        }
        if let Some(author) = author {
            args.push("--author".to_string());
            args.push(author);
        }
        args.push("-m".to_string());
        args.push(message.trim().to_string());
        run_git(&workdir, args, None)?;
        return Ok(CommitResult {
            oid: repo_head_oid(path)?,
            summary: branch_summary_core(path, false)?,
        });
    }

    let signature = repo.signature().map_err(|_| {
        GitboxError::Message("缺少 Git 用户信息，请先设置 user.name 和 user.email".to_string())
    })?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let commit_message = if sign_off {
        signed_off_message(message.trim(), &signature)
    } else {
        message.trim().to_string()
    };

    let oid = if amend {
        let current = repo
            .head()
            .map_err(|_| GitboxError::Message("当前没有可修正的提交".to_string()))?
            .peel_to_commit()?;
        let author = current.author();
        current.amend(
            Some("HEAD"),
            Some(&author),
            Some(&signature),
            None,
            Some(&commit_message),
            Some(&tree),
        )?
    } else {
        let parents = commit_parents(&repo)?;
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &parent_refs,
        )?
    };

    Ok(CommitResult {
        oid: oid.to_string(),
        summary: branch_summary_core(path, false)?,
    })
}

pub fn fetch_core(
    path: &str,
    remote_name: Option<String>,
    prune: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let name = remote_name.unwrap_or_else(|| "origin".to_string());
    let mut args = vec!["fetch".to_string()];
    if prune {
        args.push("--prune".to_string());
    }
    args.push(name.clone());
    let output = run_git(&workdir, args, None)?;

    Ok(CommandResult {
        ok: true,
        message: if prune {
            format!("已获取远程 {name} 并清理失效引用")
        } else {
            format!("已获取远程 {name}")
        },
        output,
    })
}

pub fn pull_core(path: &str, remote_name: Option<String>) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["pull".to_string(), "--ff-only".to_string()];
    if let Some(remote) = remote_name.filter(|value| !value.trim().is_empty()) {
        args.push(remote);
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: "已完成快进拉取".to_string(),
        output,
    })
}

pub fn push_with_options_core(
    path: &str,
    remote_name: Option<String>,
    target_branch: Option<String>,
    set_upstream: bool,
    force_with_lease: bool,
    push_tags: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let name = remote_name.unwrap_or_else(|| "origin".to_string());
    let head = repo.head()?;
    let branch = head
        .shorthand()
        .ok_or_else(|| GitboxError::Message("当前 HEAD 不是本地分支，不能推送".to_string()))?;
    let target = target_branch
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| branch.to_string());
    let mut args = vec!["push".to_string()];
    if set_upstream {
        args.push("-u".to_string());
    }
    if force_with_lease {
        args.push("--force-with-lease".to_string());
    }
    if push_tags {
        args.push("--tags".to_string());
    }
    args.push(name.clone());
    args.push(format!("refs/heads/{branch}:refs/heads/{target}"));
    let output = run_git(&workdir, args, None)?;

    Ok(CommandResult {
        ok: true,
        message: format!("已推送到 {name}/{target}"),
        output,
    })
}

pub fn add_remote_core(
    path: &str,
    name: String,
    url: String,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请输入远程名称")?;
    let url = clean_ref_input(url, "请输入远程 URL")?;
    let repo = Repository::discover(path)?;
    repo.remote(&name, &url)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已添加远程 {name}"),
        output: String::new(),
    })
}

pub fn update_remote_core(
    path: &str,
    name: String,
    url: String,
    push_url: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择远程")?;
    let url = clean_ref_input(url, "请输入远程 URL")?;
    let repo = Repository::discover(path)?;
    repo.remote_set_url(&name, &url)?;
    let push_url = push_url.filter(|value| !value.trim().is_empty());
    repo.remote_set_pushurl(&name, push_url.as_deref())?;
    Ok(CommandResult {
        ok: true,
        message: format!("已更新远程 {name}"),
        output: String::new(),
    })
}

pub fn delete_remote_core(path: &str, name: String) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择远程")?;
    let repo = Repository::discover(path)?;
    repo.remote_delete(&name)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已删除远程 {name}"),
        output: String::new(),
    })
}

#[allow(dead_code)]
pub fn list_commits_core(
    path: &str,
    limit: Option<usize>,
) -> Result<Vec<CommitSummary>, GitboxError> {
    list_commits_filtered_multi_core(path, limit, None, None, Vec::new(), Vec::new())
}

#[allow(dead_code)]
pub fn list_commits_filtered_core(
    path: &str,
    limit: Option<usize>,
    branch: Option<String>,
    query: Option<String>,
    author: Option<String>,
    path_filter: Option<String>,
) -> Result<Vec<CommitSummary>, GitboxError> {
    let authors = clean_optional_args(author.into_iter());
    let path_filters = clean_optional_args(path_filter.into_iter());
    list_commits_filtered_multi_core(path, limit, branch, query, authors, path_filters)
}

pub fn list_commits_filtered_multi_core(
    path: &str,
    limit: Option<usize>,
    branch: Option<String>,
    query: Option<String>,
    authors: Vec<String>,
    path_filters: Vec<String>,
) -> Result<Vec<CommitSummary>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let refs_by_oid = reference_labels(&repo)?;
    let max = limit.unwrap_or(80).clamp(1, 500);
    let query = clean_optional_arg(query);
    let authors = clean_optional_args(authors);
    let branch = clean_optional_arg(branch);
    let path_filters = clean_optional_args(path_filters);
    let scan_limit = if query.is_some() || !authors.is_empty() {
        (max * 8).clamp(max, 2500)
    } else {
        max
    };

    let mut args = vec![
        "log".to_string(),
        "--date-order".to_string(),
        "--topo-order".to_string(),
        format!("-n{scan_limit}"),
        "--format=%H".to_string(),
    ];

    if let Some(branch) = branch.as_deref() {
        args.push(branch.to_string());
    } else {
        args.push("--all".to_string());
    }

    if !path_filters.is_empty() {
        args.push("--".to_string());
        for path_filter in &path_filters {
            let rel = normalize_repo_path(&repo, path_filter)?;
            args.push(repo_path_string(&rel));
        }
    }

    let raw = run_git_raw(&workdir, args, None)?;
    if !raw.success {
        let failure = raw.failure_message();
        if failure.contains("does not have any commits")
            || failure.contains("your current branch")
            || failure.contains("unknown revision or path")
        {
            return Ok(Vec::new());
        }
        return Err(GitboxError::Message(failure));
    }

    let mut commits = Vec::new();
    let mut seen = HashMap::<Oid, bool>::new();
    for line in raw.stdout.lines() {
        let oid = match Oid::from_str(line.trim()) {
            Ok(oid) => oid,
            Err(_) => continue,
        };
        if seen.insert(oid, true).is_some() {
            continue;
        }
        let commit = repo.find_commit(oid)?;
        let refs = refs_by_oid.get(&oid).cloned().unwrap_or_default();
        if !commit_matches_any_author(&commit, &authors) {
            continue;
        }
        if !commit_matches_query(&commit, &refs, query.as_deref()) {
            continue;
        }
        commits.push(commit_summary(&commit, refs));
        if commits.len() >= max {
            break;
        }
    }
    Ok(commits)
}

pub fn commit_details_core(path: &str, oid: String) -> Result<CommitDetails, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let oid =
        Oid::from_str(oid.trim()).map_err(|_| GitboxError::Message("提交哈希无效".to_string()))?;
    let commit = repo.find_commit(oid)?;
    let refs_by_oid = reference_labels(&repo)?;
    let files_output = run_git(
        &workdir,
        vec![
            "diff-tree".to_string(),
            "--root".to_string(),
            "--no-commit-id".to_string(),
            "--name-status".to_string(),
            "-r".to_string(),
            "-M".to_string(),
            oid.to_string(),
        ],
        None,
    )?;
    let diff = run_git(
        &workdir,
        vec![
            "show".to_string(),
            "--format=".to_string(),
            "--find-renames".to_string(),
            "--patch".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            oid.to_string(),
        ],
        None,
    )?;

    Ok(CommitDetails {
        commit: commit_summary(&commit, refs_by_oid.get(&oid).cloned().unwrap_or_default()),
        files: parse_name_status(&files_output),
        diff,
    })
}

pub fn commit_file_diff_core(
    path: &str,
    oid: String,
    file_path: String,
    mode: Option<String>,
) -> Result<DiffResponse, GitboxError> {
    let oid = clean_ref_input(oid, "请选择提交")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    let diff_mode = mode.unwrap_or_else(|| "commit".to_string());

    let args = match diff_mode.as_str() {
        "commit" => vec![
            "show".to_string(),
            "--format=".to_string(),
            "--find-renames".to_string(),
            "--patch".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            oid.clone(),
            "--".to_string(),
            pathspec.clone(),
        ],
        "worktree" => vec![
            "diff".to_string(),
            "--find-renames".to_string(),
            "--patch".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            oid.clone(),
            "--".to_string(),
            pathspec.clone(),
        ],
        "parent-worktree" => vec![
            "diff".to_string(),
            "--find-renames".to_string(),
            "--patch".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            parent_or_empty_tree(&repo, &oid)?,
            "--".to_string(),
            pathspec.clone(),
        ],
        _ => return Err(GitboxError::Message("不支持的文件差异比较方式".to_string())),
    };

    let text = run_git(&workdir, args, None)?;
    Ok(DiffResponse {
        path: Some(pathspec),
        staged: false,
        hunks: parse_diff_hunks(&text),
        text,
    })
}

pub fn file_history_core(
    path: &str,
    file_path: String,
    limit: Option<usize>,
) -> Result<Vec<FileHistoryEntry>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    let max = limit.unwrap_or(80).clamp(1, 500).to_string();
    let output = run_git(
        &workdir,
        vec![
            "log".to_string(),
            "--follow".to_string(),
            format!("-n{max}"),
            "--pretty=format:%H%x09%h%x09%s%x09%an%x09%ae%x09%at".to_string(),
            "--".to_string(),
            pathspec,
        ],
        None,
    )?;
    Ok(parse_file_history(&output))
}

pub fn blame_file_core(path: &str, file_path: String) -> Result<Vec<BlameLine>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    let output = run_git(
        &workdir,
        vec![
            "blame".to_string(),
            "--line-porcelain".to_string(),
            "--".to_string(),
            pathspec,
        ],
        None,
    )?;
    Ok(parse_blame(&output))
}

pub fn compare_refs_core(
    path: &str,
    left: String,
    right: String,
) -> Result<RefComparison, GitboxError> {
    let left = clean_ref_input(left, "请选择左侧 ref")?;
    let right = clean_ref_input(right, "请选择右侧 ref")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let range = format!("{left}...{right}");
    let commit_output = run_git(
        &workdir,
        vec![
            "log".to_string(),
            "--left-right".to_string(),
            "--cherry-pick".to_string(),
            "--pretty=format:%m%x09%H%x09%h%x09%s%x09%an%x09%ae%x09%at".to_string(),
            range.clone(),
        ],
        None,
    )?;
    let file_output = run_git(
        &workdir,
        vec![
            "diff".to_string(),
            "--name-status".to_string(),
            "-M".to_string(),
            range.clone(),
        ],
        None,
    )?;
    let diff = run_git(
        &workdir,
        vec![
            "diff".to_string(),
            "--find-renames".to_string(),
            "--patch".to_string(),
            "--stat".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            range,
        ],
        None,
    )?;

    Ok(RefComparison {
        left,
        right,
        commits: parse_ref_comparison_commits(&commit_output),
        files: parse_name_status(&file_output),
        diff,
    })
}

pub fn list_branches_core(path: &str) -> Result<BranchList, GitboxError> {
    let repo = Repository::discover(path)?;
    let current = current_branch_name(&repo);
    let mut branches = Vec::new();

    collect_branches(&repo, BranchType::Local, &mut branches)?;
    collect_branches(&repo, BranchType::Remote, &mut branches)?;
    branches.sort_by(|left, right| {
        branch_type_order(&left.branch_type)
            .cmp(&branch_type_order(&right.branch_type))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut tags = Vec::new();
    for name in repo.tag_names(None)?.iter().flatten() {
        let target = repo
            .revparse_single(&format!("refs/tags/{name}"))
            .ok()
            .map(|object| object.id().to_string());
        tags.push(TagInfo {
            name: name.to_string(),
            target,
        });
    }
    tags.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(BranchList {
        current,
        branches,
        tags,
    })
}

pub fn checkout_branch_core(path: &str, name: String) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择要切换的分支")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(&workdir, vec!["checkout".to_string(), name.clone()], None)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已切换到 {name}"),
        output,
    })
}

pub fn checkout_revision_core(path: &str, revision: String) -> Result<CommandResult, GitboxError> {
    let revision = clean_ref_input(revision, "请选择要检出的 revision")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec![
            "checkout".to_string(),
            "--detach".to_string(),
            revision.clone(),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已检出 {} 为游离 HEAD", short_ref(&revision)),
        output,
    })
}

pub fn checkout_remote_branch_core(
    path: &str,
    remote_branch: String,
    local_name: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let remote_branch = clean_ref_input(remote_branch, "请选择远程分支")?;
    let local_name = local_name
        .filter(|value| !value.trim().is_empty())
        .map(|value| clean_ref_input(value, "请输入本地分支名称"))
        .transpose()?
        .unwrap_or_else(|| local_branch_from_remote(&remote_branch));
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = if repo.find_branch(&local_name, BranchType::Local).is_ok() {
        let mut output = run_git(
            &workdir,
            vec!["checkout".to_string(), local_name.clone()],
            None,
        )?;
        output.push_str(&run_git(
            &workdir,
            vec![
                "branch".to_string(),
                "--set-upstream-to".to_string(),
                remote_branch.clone(),
                local_name.clone(),
            ],
            None,
        )?);
        output
    } else {
        run_git(
            &workdir,
            vec![
                "checkout".to_string(),
                "--track".to_string(),
                "-b".to_string(),
                local_name.clone(),
                remote_branch.clone(),
            ],
            None,
        )?
    };
    Ok(CommandResult {
        ok: true,
        message: format!("已检出 {remote_branch} 为 {local_name}"),
        output,
    })
}

pub fn create_branch_core(
    path: &str,
    name: String,
    checkout: bool,
    start_point: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请输入新分支名称")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = if checkout {
        vec!["checkout".to_string(), "-b".to_string(), name.clone()]
    } else {
        vec!["branch".to_string(), name.clone()]
    };
    if let Some(start) = start_point.filter(|value| !value.trim().is_empty()) {
        args.push(start);
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: if checkout {
            format!("已创建并切换到 {name}")
        } else {
            format!("已创建分支 {name}")
        },
        output,
    })
}

pub fn rename_branch_core(
    path: &str,
    old_name: String,
    new_name: String,
) -> Result<CommandResult, GitboxError> {
    let old_name = clean_ref_input(old_name, "请选择要重命名的分支")?;
    let new_name = clean_ref_input(new_name, "请输入新分支名称")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec![
            "branch".to_string(),
            "-m".to_string(),
            old_name.clone(),
            new_name.clone(),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已将分支 {old_name} 重命名为 {new_name}"),
        output,
    })
}

pub fn cleanup_merged_branches_core(
    path: &str,
    target: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let current = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(ToOwned::to_owned));
    let target = target
        .filter(|value| !value.trim().is_empty())
        .or_else(|| current.clone())
        .unwrap_or_else(|| "HEAD".to_string());
    let merged = run_git(
        &workdir,
        vec!["branch".to_string(), "--merged".to_string(), target.clone()],
        None,
    )?;
    let protected = ["main", "master", "develop", "dev", "production"];
    let mut deleted = Vec::new();
    let mut output = String::new();
    for line in merged.lines() {
        let name = line.trim().trim_start_matches('*').trim();
        if name.is_empty()
            || current.as_deref() == Some(name)
            || protected.contains(&name)
            || name.starts_with("release/")
        {
            continue;
        }
        output.push_str(&run_git(
            &workdir,
            vec!["branch".to_string(), "-d".to_string(), name.to_string()],
            None,
        )?);
        deleted.push(name.to_string());
    }

    Ok(CommandResult {
        ok: true,
        message: if deleted.is_empty() {
            "没有可清理的已合并分支".to_string()
        } else {
            format!(
                "已清理 {} 个已合并分支：{}",
                deleted.len(),
                deleted.join(", ")
            )
        },
        output,
    })
}

pub fn set_branch_upstream_core(
    path: &str,
    branch_name: String,
    upstream: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let branch_name = clean_ref_input(branch_name, "请选择本地分支")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let upstream = upstream.filter(|value| !value.trim().is_empty());
    let has_upstream = upstream.is_some();
    let output = if let Some(upstream) = upstream {
        let upstream = clean_ref_input(upstream, "请选择上游分支")?;
        run_git(
            &workdir,
            vec![
                "branch".to_string(),
                "--set-upstream-to".to_string(),
                upstream.clone(),
                branch_name.clone(),
            ],
            None,
        )?
    } else {
        run_git(
            &workdir,
            vec![
                "branch".to_string(),
                "--unset-upstream".to_string(),
                branch_name.clone(),
            ],
            None,
        )?
    };
    Ok(CommandResult {
        ok: true,
        message: if has_upstream {
            format!("已设置 {branch_name} 的上游分支")
        } else {
            format!("已取消 {branch_name} 的上游分支")
        },
        output,
    })
}

pub fn delete_branch_core(
    path: &str,
    name: String,
    force: bool,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择要删除的分支")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let delete_flag = if force { "-D" } else { "-d" };
    let output = run_git(
        &workdir,
        vec!["branch".to_string(), delete_flag.to_string(), name.clone()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已删除分支 {name}"),
        output,
    })
}

pub fn delete_remote_branch_core(
    path: &str,
    remote_branch: String,
) -> Result<CommandResult, GitboxError> {
    let remote_branch = clean_ref_input(remote_branch, "请选择要删除的远程分支")?;
    let (remote, branch) = split_remote_branch(&remote_branch)?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut output = run_git(
        &workdir,
        vec![
            "push".to_string(),
            remote.clone(),
            "--delete".to_string(),
            branch.clone(),
        ],
        None,
    )?;
    let delete_tracking = run_git_raw(
        &workdir,
        vec![
            "branch".to_string(),
            "-dr".to_string(),
            remote_branch.clone(),
        ],
        None,
    )?;
    output.push_str(&delete_tracking.combined_output());
    Ok(CommandResult {
        ok: true,
        message: format!("已删除远程分支 {remote_branch}"),
        output,
    })
}

pub fn create_tag_core(
    path: &str,
    name: String,
    target: Option<String>,
    annotated: bool,
    message: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请输入标签名称")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let target = target
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "HEAD".to_string());
    let mut args = vec!["tag".to_string()];
    if annotated {
        args.push("-a".to_string());
        args.push(name.clone());
        args.push(target.clone());
        args.push("-m".to_string());
        args.push(
            message
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| name.clone()),
        );
    } else {
        args.push(name.clone());
        args.push(target.clone());
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已创建标签 {name}"),
        output,
    })
}

pub fn delete_tag_core(path: &str, name: String) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择要删除的标签")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec!["tag".to_string(), "-d".to_string(), name.clone()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已删除本地标签 {name}"),
        output,
    })
}

pub fn push_tag_core(
    path: &str,
    remote_name: Option<String>,
    name: String,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择要推送的标签")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let remote = remote_name.unwrap_or_else(|| "origin".to_string());
    let output = run_git(
        &workdir,
        vec![
            "push".to_string(),
            remote.clone(),
            format!("refs/tags/{name}:refs/tags/{name}"),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已推送标签 {name} 到 {remote}"),
        output,
    })
}

pub fn delete_remote_tag_core(
    path: &str,
    remote_name: Option<String>,
    name: String,
) -> Result<CommandResult, GitboxError> {
    let name = clean_ref_input(name, "请选择要删除的远程标签")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let remote = remote_name.unwrap_or_else(|| "origin".to_string());
    let output = run_git(
        &workdir,
        vec![
            "push".to_string(),
            remote.clone(),
            format!(":refs/tags/{name}"),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已删除 {remote} 上的标签 {name}"),
        output,
    })
}

pub fn merge_branch_core(
    path: &str,
    target: String,
    no_ff: bool,
    no_commit: bool,
    squash: bool,
) -> Result<CommandResult, GitboxError> {
    let target = clean_ref_input(target, "请选择要合并的分支")?;
    let mut args = vec!["merge".to_string()];
    if no_ff {
        args.push("--no-ff".to_string());
    }
    if no_commit {
        args.push("--no-commit".to_string());
    }
    if squash {
        args.push("--squash".to_string());
    }
    args.push(target.clone());
    run_git_operation(
        path,
        args,
        format!("已合并 {target}"),
        format!("合并 {target} 时产生冲突，请解决后继续"),
    )
}

pub fn rebase_branch_core(
    path: &str,
    target: String,
    autostash: bool,
) -> Result<CommandResult, GitboxError> {
    let target = clean_ref_input(target, "请选择变基目标分支")?;
    let mut args = vec!["rebase".to_string()];
    if autostash {
        args.push("--autostash".to_string());
    }
    args.push(target.clone());
    run_git_operation(
        path,
        args,
        format!("已变基到 {target}"),
        format!("变基到 {target} 时产生冲突，请解决后继续"),
    )
}

pub fn rebase_advanced_core(
    path: &str,
    target: Option<String>,
    source_branch: Option<String>,
    onto: Option<String>,
    autostash: bool,
    interactive: bool,
    autosquash: bool,
    rebase_merges: bool,
    keep_empty: bool,
    root: bool,
    update_refs: bool,
) -> Result<CommandResult, GitboxError> {
    let target = clean_optional_arg(target);
    let source_branch = clean_optional_arg(source_branch);
    let onto = clean_optional_arg(onto);
    if !root && target.is_none() {
        return Err(GitboxError::Message(
            "请选择变基目标或启用“从根提交”".to_string(),
        ));
    }
    if onto.is_some() && target.is_none() {
        return Err(GitboxError::Message(
            "指定新基线时需要同时指定上游起点".to_string(),
        ));
    }

    let mut args = vec!["rebase".to_string()];
    if interactive {
        args.push("--interactive".to_string());
    }
    if autostash {
        args.push("--autostash".to_string());
    }
    if autosquash {
        args.push("--autosquash".to_string());
    }
    if rebase_merges {
        args.push("--rebase-merges".to_string());
    }
    if keep_empty {
        args.push("--keep-empty".to_string());
    }
    if update_refs {
        args.push("--update-refs".to_string());
    }
    if root {
        args.push("--root".to_string());
    }
    if let Some(onto) = onto.as_deref() {
        args.push("--onto".to_string());
        args.push(onto.to_string());
    }
    if let Some(target) = target.as_deref() {
        args.push(target.to_string());
    }
    if let Some(source) = source_branch.as_deref() {
        args.push(source.to_string());
    }

    let target_label = target
        .as_deref()
        .or(onto.as_deref())
        .unwrap_or("--root")
        .to_string();
    run_git_operation(
        path,
        args,
        format!("已执行高级变基到 {target_label}"),
        format!("高级变基到 {target_label} 时产生冲突，请解决后继续"),
    )
}

pub fn cherry_pick_commit_core(path: &str, oid: String) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要挑选的提交")?;
    run_git_operation(
        path,
        vec!["cherry-pick".to_string(), oid.clone()],
        format!("已挑选提交 {}", short_ref(&oid)),
        format!("挑选提交 {} 时产生冲突，请解决后继续", short_ref(&oid)),
    )
}

pub fn revert_commit_core(
    path: &str,
    oid: String,
    no_commit: bool,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要反向提交的提交")?;
    let mut args = vec!["revert".to_string(), "--no-edit".to_string()];
    if no_commit {
        args.push("--no-commit".to_string());
    }
    args.push(oid.clone());
    run_git_operation(
        path,
        args,
        format!("已反向提交 {}", short_ref(&oid)),
        format!("反向提交 {} 时产生冲突，请解决后继续", short_ref(&oid)),
    )
}

pub fn reset_to_commit_core(
    path: &str,
    oid: String,
    mode: String,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要重置到的提交")?;
    let mode = match mode.as_str() {
        "soft" => "--soft",
        "mixed" => "--mixed",
        "hard" => "--hard",
        _ => {
            return Err(GitboxError::Message(
                "重置模式只支持软重置、混合重置或硬重置".to_string(),
            ))
        }
    };
    let mode_label = match mode {
        "--soft" => "软重置",
        "--mixed" => "混合重置",
        "--hard" => "硬重置",
        _ => "重置",
    };
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec!["reset".to_string(), mode.to_string(), oid.clone()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已{mode_label}到 {}", short_ref(&oid)),
        output,
    })
}

pub fn undo_last_commit_core(path: &str, keep_staged: bool) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let head = repo.head()?.peel_to_commit()?;
    if head.parent_count() == 0 {
        return Err(GitboxError::Message(
            "当前提交没有父提交，不能撤销最后一次提交".to_string(),
        ));
    }
    let workdir = repo_workdir(&repo)?;
    let mode = if keep_staged { "--soft" } else { "--mixed" };
    let output = run_git(
        &workdir,
        vec!["reset".to_string(), mode.to_string(), "HEAD~1".to_string()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: if keep_staged {
            "已撤销最后一次提交，变更保留在暂存区".to_string()
        } else {
            "已撤销最后一次提交，变更保留在工作区".to_string()
        },
        output,
    })
}

pub fn fixup_commit_core(
    path: &str,
    oid: String,
    squash: bool,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择目标提交")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mode = if squash { "--squash" } else { "--fixup" };
    let output = run_git(
        &workdir,
        vec![
            "commit".to_string(),
            mode.to_string(),
            oid.clone(),
            "--no-verify".to_string(),
        ],
        None,
    )?;
    let new_head = repo_head_oid(path)?;
    Ok(CommandResult {
        ok: true,
        message: if squash {
            format!("已创建压缩提交，目标 {}", short_ref(&oid))
        } else {
            format!("已创建修正提交，目标 {}", short_ref(&oid))
        },
        output: if output.trim().is_empty() {
            new_head
        } else {
            output
        },
    })
}

pub fn drop_commit_core(path: &str, oid: String) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要丢弃的提交")?;
    let parent = format!("{oid}^");
    let args = vec![
        "rebase".to_string(),
        "--onto".to_string(),
        parent.clone(),
        oid.clone(),
    ];
    run_git_operation(
        path,
        args,
        format!("已丢弃提交 {}", short_ref(&oid)),
        format!("丢弃提交 {} 时产生冲突，请解决后继续", short_ref(&oid)),
    )
}

pub fn push_commit_core(
    path: &str,
    remote_name: Option<String>,
    oid: String,
    target_branch: Option<String>,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要推送的提交")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let remote = remote_name.unwrap_or_else(|| "origin".to_string());
    let target = target_branch
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            repo.head()
                .ok()
                .and_then(|head| head.shorthand().map(ToOwned::to_owned))
        })
        .ok_or_else(|| GitboxError::Message("请输入目标分支".to_string()))?;
    let output = run_git(
        &workdir,
        vec![
            "push".to_string(),
            remote.clone(),
            format!("{oid}:refs/heads/{target}"),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已将 {} 推送到 {remote}/{target}", short_ref(&oid)),
        output,
    })
}

pub fn create_patch_core(
    path: &str,
    paths: Vec<String>,
    staged: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let paths_empty = paths.is_empty();
    let mut regular_paths = Vec::new();
    let mut output = String::new();

    if !staged {
        for path in &paths {
            let rel = normalize_repo_path(&repo, path)?;
            let pathspec = repo_path_string(&rel);
            let status = repo
                .status_file(Path::new(&pathspec))
                .unwrap_or(Status::WT_NEW);
            if status.contains(Status::WT_NEW) && !has_index_change(status) {
                let raw = run_git_raw(
                    &workdir,
                    vec![
                        "diff".to_string(),
                        "--no-index".to_string(),
                        "--binary".to_string(),
                        "--no-color".to_string(),
                        "--".to_string(),
                        "/dev/null".to_string(),
                        pathspec,
                    ],
                    None,
                )?;
                if raw.stdout.trim().is_empty() && !raw.success {
                    return Err(GitboxError::Message(raw.failure_message()));
                }
                output.push_str(&raw.stdout);
            } else {
                regular_paths.push(path.clone());
            }
        }
    } else {
        regular_paths = paths;
    }

    let mut args = vec![
        "diff".to_string(),
        "--binary".to_string(),
        "--patch".to_string(),
        "--no-ext-diff".to_string(),
        "--no-color".to_string(),
    ];
    if staged {
        args.push("--cached".to_string());
    }
    if !regular_paths.is_empty() {
        args.push("--".to_string());
        args.extend(regular_paths);
    }
    if staged || !args.last().is_some_and(|arg| arg == "--no-color") {
        output.push_str(&run_git(&workdir, args, None)?);
    } else if paths_empty {
        output.push_str(&run_git(&workdir, args, None)?);
    }
    Ok(CommandResult {
        ok: true,
        message: if output.trim().is_empty() {
            "没有可生成补丁的差异".to_string()
        } else {
            "已生成补丁".to_string()
        },
        output,
    })
}

pub fn apply_patch_core(
    path: &str,
    patch: String,
    index: bool,
    three_way: bool,
) -> Result<CommandResult, GitboxError> {
    if patch.trim().is_empty() {
        return Err(GitboxError::Message("请输入补丁内容".to_string()));
    }
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["apply".to_string(), "--whitespace=nowarn".to_string()];
    if three_way {
        args.push("--3way".to_string());
    }
    if index {
        args.push("--index".to_string());
    }
    args.push("-".to_string());
    let output = run_git(&workdir, args, Some(&patch))?;
    Ok(CommandResult {
        ok: true,
        message: if index {
            "已应用补丁并更新索引".to_string()
        } else {
            "已应用补丁到工作区".to_string()
        },
        output,
    })
}

pub fn cherry_pick_files_core(
    path: &str,
    oid: String,
    files: Vec<String>,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要应用的提交")?;
    if files.is_empty() {
        return Err(GitboxError::Message("请选择要应用的文件".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut show_args = vec![
        "show".to_string(),
        "--format=".to_string(),
        "--find-renames".to_string(),
        "--patch".to_string(),
        "--no-ext-diff".to_string(),
        "--no-color".to_string(),
        oid.clone(),
        "--".to_string(),
    ];
    show_args.extend(files);
    let patch = run_git(&workdir, show_args, None)?;
    if patch.trim().is_empty() {
        return Err(GitboxError::Message(
            "所选文件在该提交中没有可应用的变更".to_string(),
        ));
    }

    let raw = run_git_raw(
        &workdir,
        vec![
            "apply".to_string(),
            "--3way".to_string(),
            "--whitespace=nowarn".to_string(),
            "-".to_string(),
        ],
        Some(&patch),
    )?;
    if raw.success {
        return Ok(CommandResult {
            ok: true,
            message: format!("已应用 {} 的所选文件变更", short_ref(&oid)),
            output: raw.combined_output(),
        });
    }

    let state = operation_state_core(path)?;
    if !state.conflicted_paths.is_empty() {
        Ok(CommandResult {
            ok: false,
            message: format!(
                "应用 {} 的所选文件时产生冲突，请解决后继续",
                short_ref(&oid)
            ),
            output: raw.combined_output(),
        })
    } else {
        Err(GitboxError::Message(raw.failure_message()))
    }
}

pub fn revert_commit_files_core(
    path: &str,
    oid: String,
    files: Vec<String>,
) -> Result<CommandResult, GitboxError> {
    let oid = clean_ref_input(oid, "请选择要还原的提交")?;
    if files.is_empty() {
        return Err(GitboxError::Message("请选择要还原的文件".to_string()));
    }

    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let normalized_files = files
        .iter()
        .map(|file| normalize_repo_path(&repo, file).map(|path| repo_path_string(&path)))
        .collect::<Result<Vec<_>, _>>()?;

    let mut show_args = vec![
        "show".to_string(),
        "--format=".to_string(),
        "--find-renames".to_string(),
        "--patch".to_string(),
        "--no-ext-diff".to_string(),
        "--no-color".to_string(),
        oid.clone(),
        "--".to_string(),
    ];
    show_args.extend(normalized_files);
    let patch = run_git(&workdir, show_args, None)?;
    if patch.trim().is_empty() {
        return Err(GitboxError::Message(
            "所选文件在该提交中没有可还原的变更".to_string(),
        ));
    }

    let raw = run_git_raw(
        &workdir,
        vec![
            "apply".to_string(),
            "-R".to_string(),
            "--3way".to_string(),
            "--whitespace=nowarn".to_string(),
            "-".to_string(),
        ],
        Some(&patch),
    )?;
    if raw.success {
        return Ok(CommandResult {
            ok: true,
            message: format!("已还原 {} 的所选文件变更", short_ref(&oid)),
            output: raw.combined_output(),
        });
    }

    Err(GitboxError::Message(raw.failure_message()))
}

pub fn operation_state_core(path: &str) -> Result<GitOperationState, GitboxError> {
    let repo = Repository::discover(path)?;
    let git_dir = repo.path();
    let operation =
        if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
            Some("rebase".to_string())
        } else if git_dir.join("MERGE_HEAD").exists() {
            Some("merge".to_string())
        } else if git_dir.join("CHERRY_PICK_HEAD").exists() {
            Some("cherry-pick".to_string())
        } else if git_dir.join("REVERT_HEAD").exists() {
            Some("revert".to_string())
        } else {
            None
        };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let mut conflicted_paths = Vec::new();
    for entry in repo.statuses(Some(&mut opts))?.iter() {
        if !entry.status().contains(Status::CONFLICTED) {
            continue;
        }
        if let Some(path) = entry.path() {
            conflicted_paths.push(path.to_string());
        }
    }
    conflicted_paths.sort();

    Ok(GitOperationState {
        active: operation.is_some() || !conflicted_paths.is_empty(),
        operation,
        conflicted_paths,
    })
}

pub fn operation_control_core(path: &str, action: String) -> Result<CommandResult, GitboxError> {
    let action = clean_ref_input(action, "请选择操作")?;
    let state = operation_state_core(path)?;
    let operation = state
        .operation
        .as_deref()
        .ok_or_else(|| GitboxError::Message("当前没有进行中的 Git 操作".to_string()))?;

    let args = match (operation, action.as_str()) {
        ("rebase", "continue") => vec!["rebase".to_string(), "--continue".to_string()],
        ("rebase", "abort") => vec!["rebase".to_string(), "--abort".to_string()],
        ("rebase", "skip") => vec!["rebase".to_string(), "--skip".to_string()],
        ("merge", "continue") => vec!["commit".to_string(), "--no-edit".to_string()],
        ("merge", "abort") => vec!["merge".to_string(), "--abort".to_string()],
        ("cherry-pick", "continue") => vec!["cherry-pick".to_string(), "--continue".to_string()],
        ("cherry-pick", "abort") => vec!["cherry-pick".to_string(), "--abort".to_string()],
        ("cherry-pick", "skip") => vec!["cherry-pick".to_string(), "--skip".to_string()],
        ("revert", "continue") => vec!["revert".to_string(), "--continue".to_string()],
        ("revert", "abort") => vec!["revert".to_string(), "--abort".to_string()],
        ("revert", "skip") => vec!["revert".to_string(), "--skip".to_string()],
        _ => {
            return Err(GitboxError::Message(format!(
                "{} 不支持“{}”操作",
                operation_label(operation),
                operation_action_label(action.as_str())
            )))
        }
    };

    run_git_operation(
        path,
        args,
        format!(
            "已执行{}的{}操作",
            operation_label(operation),
            operation_action_label(action.as_str())
        ),
        format!(
            "{}{}后仍有冲突，请继续处理",
            operation_label(operation),
            operation_action_label(action.as_str())
        ),
    )
}

pub fn conflict_details_core(
    path: &str,
    file_path: String,
) -> Result<ConflictDetails, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    let current = fs::read_to_string(workdir.join(&rel)).ok();
    let blocks = current
        .as_deref()
        .map(parse_conflict_blocks_for_response)
        .unwrap_or_default();

    Ok(ConflictDetails {
        path: pathspec.clone(),
        base: read_index_stage(&workdir, 1, &pathspec)?,
        ours: read_index_stage(&workdir, 2, &pathspec)?,
        theirs: read_index_stage(&workdir, 3, &pathspec)?,
        current,
        blocks,
    })
}

pub fn resolve_conflict_file_core(
    path: &str,
    file_path: String,
    side: String,
) -> Result<CommandResult, GitboxError> {
    let side = conflict_side_flag(&side)?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    run_git(
        &workdir,
        git_args_with_paths(&["checkout", side], std::slice::from_ref(&pathspec)),
        None,
    )?;
    run_git(
        &workdir,
        git_args_with_paths(&["add"], std::slice::from_ref(&pathspec)),
        None,
    )?;

    Ok(CommandResult {
        ok: true,
        message: format!("已用{}解决 {pathspec}", conflict_side_label(side)),
        output: String::new(),
    })
}

pub fn resolve_conflict_block_core(
    path: &str,
    file_path: String,
    block_index: usize,
    side: String,
) -> Result<CommandResult, GitboxError> {
    let side = clean_ref_input(side, "请选择冲突处理方式")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let full_path = workdir.join(&rel);
    let content = fs::read_to_string(&full_path)?;
    let lines = split_preserving_newlines(&content);
    let blocks = parse_conflict_blocks(&lines);
    let block = blocks
        .iter()
        .find(|block| block.index == block_index)
        .ok_or_else(|| GitboxError::Message("没有找到指定冲突块".to_string()))?;
    let replacement = match side.as_str() {
        "ours" => &block.ours,
        "base" => &block.base,
        "theirs" => &block.theirs,
        _ => {
            return Err(GitboxError::Message(
                "冲突块处理方式只支持当前、基线或传入".to_string(),
            ))
        }
    };

    let mut next = Vec::new();
    next.extend_from_slice(&lines[..block.start]);
    next.extend(replacement.iter().cloned());
    next.extend_from_slice(&lines[block.end + 1..]);
    let next_content = next.concat();
    fs::write(&full_path, next_content.as_bytes())?;

    if !next_content.contains("<<<<<<< ") {
        let pathspec = repo_path_string(&rel);
        run_git(
            &workdir,
            git_args_with_paths(&["add"], std::slice::from_ref(&pathspec)),
            None,
        )?;
    }

    Ok(CommandResult {
        ok: true,
        message: format!(
            "已处理 {} 的第 {} 个冲突块",
            repo_path_string(&rel),
            block_index + 1
        ),
        output: String::new(),
    })
}

pub fn mark_conflict_resolved_core(
    path: &str,
    file_path: String,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);
    let current = fs::read_to_string(workdir.join(&rel)).unwrap_or_default();
    if current.contains("<<<<<<< ") {
        return Err(GitboxError::Message(
            "文件仍包含冲突标记，不能标记为已解决".to_string(),
        ));
    }
    run_git(
        &workdir,
        git_args_with_paths(&["add"], std::slice::from_ref(&pathspec)),
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已标记 {pathspec} 为解决"),
        output: String::new(),
    })
}

pub fn save_conflict_result_core(
    path: &str,
    file_path: String,
    content: String,
    mark_resolved: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let rel = normalize_repo_path(&repo, &file_path)?;
    let pathspec = repo_path_string(&rel);

    if mark_resolved {
        if content.contains("<<<<<<< ")
            || content.contains("=======")
            || content.contains(">>>>>>> ")
        {
            return Err(GitboxError::Message(
                "结果仍包含冲突标记，不能标记为已解决".to_string(),
            ));
        }
    }

    fs::write(workdir.join(&rel), content.as_bytes())?;

    if mark_resolved {
        run_git(
            &workdir,
            git_args_with_paths(&["add"], std::slice::from_ref(&pathspec)),
            None,
        )?;
    }

    Ok(CommandResult {
        ok: true,
        message: if mark_resolved {
            format!("已保存并标记 {pathspec} 为解决")
        } else {
            format!("已保存 {pathspec} 的合并结果")
        },
        output: String::new(),
    })
}

pub fn list_worktrees_core(path: &str) -> Result<Vec<WorktreeInfo>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec![
            "worktree".to_string(),
            "list".to_string(),
            "--porcelain".to_string(),
        ],
        None,
    )?;
    Ok(parse_worktrees(&output))
}

pub fn create_worktree_core(
    path: &str,
    worktree_path: String,
    branch: Option<String>,
    start_point: Option<String>,
    detach: bool,
) -> Result<CommandResult, GitboxError> {
    let worktree_path = clean_ref_input(worktree_path, "请输入工作树目录")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["worktree".to_string(), "add".to_string()];
    if detach {
        args.push("--detach".to_string());
    } else if let Some(branch) = branch.filter(|value| !value.trim().is_empty()) {
        args.push("-b".to_string());
        args.push(branch);
    }
    args.push(worktree_path.clone());
    if let Some(start) = start_point.filter(|value| !value.trim().is_empty()) {
        args.push(start);
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已创建工作树 {worktree_path}"),
        output,
    })
}

pub fn remove_worktree_core(
    path: &str,
    worktree_path: String,
    force: bool,
) -> Result<CommandResult, GitboxError> {
    let worktree_path = clean_ref_input(worktree_path, "请选择工作树")?;
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["worktree".to_string(), "remove".to_string()];
    if force {
        args.push("--force".to_string());
    }
    args.push(worktree_path.clone());
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: format!("已移除工作树 {worktree_path}"),
        output,
    })
}

pub fn list_stashes_core(path: &str) -> Result<Vec<StashInfo>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec![
            "stash".to_string(),
            "list".to_string(),
            "--format=%gd%x09%H%x09%ct%x09%s".to_string(),
        ],
        None,
    )?;
    Ok(parse_stashes(&output))
}

pub fn stash_action_core(
    path: &str,
    stash_ref: String,
    action: String,
) -> Result<CommandResult, GitboxError> {
    let stash_ref = clean_ref_input(stash_ref, "请选择贮藏记录")?;
    let action = clean_ref_input(action, "请选择贮藏操作")?;
    let subcommand = match action.as_str() {
        "apply" => "apply",
        "pop" => "pop",
        "drop" => "drop",
        _ => {
            return Err(GitboxError::Message(
                "贮藏操作只支持应用、弹出或删除".to_string(),
            ))
        }
    };
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec![
            "stash".to_string(),
            subcommand.to_string(),
            stash_ref.clone(),
        ],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: format!("已{}贮藏记录 {stash_ref}", stash_action_label(subcommand)),
        output,
    })
}

pub fn clear_stashes_core(path: &str) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec!["stash".to_string(), "clear".to_string()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: "已清空贮藏记录".to_string(),
        output,
    })
}

pub fn list_submodules_core(path: &str) -> Result<Vec<SubmoduleInfo>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git_raw(
        &workdir,
        vec![
            "submodule".to_string(),
            "status".to_string(),
            "--recursive".to_string(),
        ],
        None,
    )?;
    if !output.success && output.failure_message().contains("no submodule mapping") {
        return Ok(Vec::new());
    }
    if !output.success {
        return Err(GitboxError::Message(output.failure_message()));
    }
    Ok(parse_submodules(&output.stdout))
}

pub fn update_submodules_core(
    path: &str,
    init: bool,
    recursive: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec!["submodule".to_string(), "update".to_string()];
    if init {
        args.push("--init".to_string());
    }
    if recursive {
        args.push("--recursive".to_string());
    }
    let output = run_git(&workdir, args, None)?;
    Ok(CommandResult {
        ok: true,
        message: "已更新子模块".to_string(),
        output,
    })
}

pub fn lfs_status_core(path: &str) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git(
        &workdir,
        vec!["lfs".to_string(), "status".to_string()],
        None,
    )?;
    Ok(CommandResult {
        ok: true,
        message: "已读取 Git LFS 状态".to_string(),
        output,
    })
}

pub fn commit_message_history_core(
    path: &str,
    limit: Option<usize>,
) -> Result<Vec<String>, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let max = limit.unwrap_or(30).clamp(1, 200).to_string();
    let output = run_git(
        &workdir,
        vec![
            "log".to_string(),
            format!("-n{max}"),
            "--format=%s".to_string(),
        ],
        None,
    )?;
    let mut messages = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    messages.dedup();
    Ok(messages)
}

pub fn record_recent_repo(app: &AppHandle, info: &RepositoryInfo) -> Result<(), GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    let name = Path::new(&info.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("仓库")
        .to_string();
    conn.execute(
        "INSERT INTO recent_repositories(path, name, branch, last_opened_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET
           name = excluded.name,
           branch = excluded.branch,
           last_opened_at = excluded.last_opened_at",
        params![info.path, name, info.branch, now_unix()],
    )?;
    Ok(())
}

pub fn record_shelf(
    app: &AppHandle,
    repo_path: &str,
    draft: ShelfDraft,
) -> Result<ShelfInfo, GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    conn.execute(
        "INSERT INTO shelves(repo_path, message, stash_ref, stash_oid, created_at, applied_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
        params![
            repo_path,
            draft.message,
            draft.stash_ref,
            draft.stash_oid,
            draft.created_at
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(ShelfInfo {
        id: Some(id),
        repo_path: repo_path.to_string(),
        message: draft.message,
        stash_ref: draft.stash_ref,
        stash_oid: draft.stash_oid,
        created_at: draft.created_at,
        applied_at: None,
    })
}

pub fn mark_shelf_applied(app: &AppHandle, stash_ref: &str) -> Result<(), GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    conn.execute(
        "UPDATE shelves SET applied_at = ?1 WHERE stash_ref = ?2 AND applied_at IS NULL",
        params![now_unix(), stash_ref],
    )?;
    Ok(())
}

pub fn list_shelves_core(app: &AppHandle, repo_path: &str) -> Result<Vec<ShelfInfo>, GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, message, stash_ref, stash_oid, created_at, applied_at
         FROM shelves
         WHERE repo_path = ?1
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map(params![repo_path], |row| {
        Ok(ShelfInfo {
            id: row.get(0)?,
            repo_path: row.get(1)?,
            message: row.get(2)?,
            stash_ref: row.get(3)?,
            stash_oid: row.get(4)?,
            created_at: row.get(5)?,
            applied_at: row.get(6)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn repository_info(repo: &Repository) -> Result<RepositoryInfo, GitboxError> {
    let workdir = repo.workdir().map(path_string);
    let root = repo_root(repo)?;
    let branch = current_branch_name(repo);
    let head = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string());

    Ok(RepositoryInfo {
        path: path_string(&root),
        workdir,
        git_dir: path_string(repo.path()),
        is_bare: repo.is_bare(),
        branch,
        head,
        remotes: remote_infos(repo)?,
    })
}

fn current_branch_name(repo: &Repository) -> Option<String> {
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            return Some(name.to_string());
        }
    }

    let head_path = repo.path().join("HEAD");
    let content = fs::read_to_string(head_path).ok()?;
    content
        .trim()
        .strip_prefix("ref: refs/heads/")
        .map(ToOwned::to_owned)
}

fn remote_infos(repo: &Repository) -> Result<Vec<RemoteInfo>, GitboxError> {
    let names = repo.remotes()?;
    let mut remotes = Vec::new();
    for name in names.iter().flatten() {
        let remote = repo.find_remote(name)?;
        remotes.push(RemoteInfo {
            name: name.to_string(),
            url: remote.url().map(ToOwned::to_owned),
            push_url: remote.pushurl().map(ToOwned::to_owned),
        });
    }
    Ok(remotes)
}

fn reference_labels(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>, GitboxError> {
    let mut refs_by_oid: HashMap<Oid, Vec<String>> = HashMap::new();
    for reference in repo.references()? {
        let reference = reference?;
        if reference.name() == Some("refs/remotes/origin/HEAD") {
            continue;
        }
        if let Some(oid) = reference.target() {
            let label = reference
                .shorthand()
                .or_else(|| reference.name())
                .unwrap_or_default()
                .to_string();
            if !label.is_empty() {
                refs_by_oid.entry(oid).or_default().push(label);
            }
        }
    }

    for labels in refs_by_oid.values_mut() {
        labels.sort();
        labels.dedup();
    }
    Ok(refs_by_oid)
}

fn commit_summary(commit: &git2::Commit<'_>, refs: Vec<String>) -> CommitSummary {
    let author = commit.author();
    let committer = commit.committer();
    CommitSummary {
        oid: commit.id().to_string(),
        short_oid: commit.id().to_string().chars().take(10).collect(),
        summary: commit.summary().unwrap_or("(无提交信息)").to_string(),
        body: commit.body().unwrap_or_default().to_string(),
        author_name: author.name().unwrap_or("Unknown").to_string(),
        author_email: author.email().unwrap_or_default().to_string(),
        author_time: author.when().seconds(),
        committer_time: committer.when().seconds(),
        parents: commit.parent_ids().map(|oid| oid.to_string()).collect(),
        refs,
    }
}

fn parse_name_status(output: &str) -> Vec<CommitFileChange> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() < 2 {
                return None;
            }
            let status = parts.remove(0).to_string();
            let (old_path, path) = if status.starts_with('R') || status.starts_with('C') {
                if parts.len() < 2 {
                    (None, parts.first().unwrap_or(&"").to_string())
                } else {
                    (Some(parts[0].to_string()), parts[1].to_string())
                }
            } else {
                (None, parts[0].to_string())
            };
            if path.is_empty() {
                return None;
            }
            Some(CommitFileChange {
                path,
                old_path,
                status,
            })
        })
        .collect()
}

fn parse_file_history(output: &str) -> Vec<FileHistoryEntry> {
    output
        .lines()
        .filter_map(|line| {
            let parts = line.splitn(6, '\t').collect::<Vec<_>>();
            if parts.len() < 6 {
                return None;
            }
            Some(FileHistoryEntry {
                oid: parts[0].to_string(),
                short_oid: parts[1].to_string(),
                summary: parts[2].to_string(),
                author_name: parts[3].to_string(),
                author_email: parts[4].to_string(),
                author_time: parts[5].parse().unwrap_or_default(),
            })
        })
        .collect()
}

fn parse_ref_comparison_commits(output: &str) -> Vec<RefComparisonCommit> {
    output
        .lines()
        .filter_map(|line| {
            let parts = line.splitn(7, '\t').collect::<Vec<_>>();
            if parts.len() < 7 {
                return None;
            }
            let side = match parts[0] {
                "<" => "left",
                ">" => "right",
                _ => "both",
            };
            Some(RefComparisonCommit {
                side: side.to_string(),
                oid: parts[1].to_string(),
                short_oid: parts[2].to_string(),
                summary: parts[3].to_string(),
                author_name: parts[4].to_string(),
                author_email: parts[5].to_string(),
                author_time: parts[6].parse().unwrap_or_default(),
            })
        })
        .collect()
}

fn parse_blame(output: &str) -> Vec<BlameLine> {
    let mut lines = Vec::new();
    let mut oid = String::new();
    let mut line_number = 0usize;
    let mut author_name = String::new();
    let mut author_email = String::new();
    let mut author_time = 0i64;
    let mut summary = String::new();

    for line in output.lines() {
        if let Some(content) = line.strip_prefix('\t') {
            lines.push(BlameLine {
                line_number,
                short_oid: short_ref(&oid),
                oid: oid.clone(),
                author_name: author_name.clone(),
                author_email: author_email.clone(),
                author_time,
                summary: summary.clone(),
                content: content.to_string(),
            });
            continue;
        }

        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() >= 3
            && parts[0].len() == 40
            && parts[0].chars().all(|c| c.is_ascii_hexdigit())
        {
            oid = parts[0].to_string();
            line_number = parts[2].parse().unwrap_or_default();
            author_name.clear();
            author_email.clear();
            author_time = 0;
            summary.clear();
            continue;
        }

        if let Some(value) = line.strip_prefix("author ") {
            author_name = value.to_string();
        } else if let Some(value) = line.strip_prefix("author-mail ") {
            author_email = value.trim_matches(['<', '>']).to_string();
        } else if let Some(value) = line.strip_prefix("author-time ") {
            author_time = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("summary ") {
            summary = value.to_string();
        }
    }

    lines
}

fn parse_worktrees(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current: Option<WorktreeInfo> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(item) = current.take() {
                worktrees.push(item);
            }
            current = Some(WorktreeInfo {
                path: path.to_string(),
                head: None,
                branch: None,
                detached: false,
                bare: false,
                prunable: false,
            });
        } else if let Some(item) = current.as_mut() {
            if let Some(head) = line.strip_prefix("HEAD ") {
                item.head = Some(head.to_string());
            } else if let Some(branch) = line.strip_prefix("branch ") {
                item.branch = Some(format_ref_label(branch));
            } else if line == "detached" {
                item.detached = true;
            } else if line == "bare" {
                item.bare = true;
            } else if line.starts_with("prunable") {
                item.prunable = true;
            }
        }
    }

    if let Some(item) = current {
        worktrees.push(item);
    }
    worktrees
}

fn parse_stashes(output: &str) -> Vec<StashInfo> {
    output
        .lines()
        .filter_map(|line| {
            let parts = line.splitn(4, '\t').collect::<Vec<_>>();
            if parts.len() < 4 {
                return None;
            }
            Some(StashInfo {
                stash_ref: parts[0].to_string(),
                oid: parts[1].to_string(),
                created_at: parts[2].parse().unwrap_or_default(),
                message: parts[3].to_string(),
            })
        })
        .collect()
}

fn parse_submodules(output: &str) -> Vec<SubmoduleInfo> {
    output
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            let status_char = line.chars().next().unwrap_or(' ');
            let rest = line.get(1..)?.trim();
            let mut parts = rest.split_whitespace();
            let oid = parts
                .next()?
                .trim_start_matches('-')
                .trim_start_matches('+')
                .to_string();
            let path = parts.next()?.to_string();
            let branch = parts
                .next()
                .map(|value| value.trim_matches(['(', ')']).to_string())
                .filter(|value| !value.is_empty());
            let status = match status_char {
                '-' => "not_initialized",
                '+' => "different_head",
                'U' => "conflicted",
                _ => "clean",
            };
            Some(SubmoduleInfo {
                path,
                oid,
                status: status.to_string(),
                branch,
            })
        })
        .collect()
}

fn repo_head_oid(path: &str) -> Result<String, GitboxError> {
    let repo = Repository::discover(path)?;
    Ok(repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string())
        .unwrap_or_default())
}

fn collect_branches(
    repo: &Repository,
    branch_type: BranchType,
    output: &mut Vec<BranchInfo>,
) -> Result<(), GitboxError> {
    for branch in repo.branches(Some(branch_type))? {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap_or_default().to_string();
        if name.is_empty() || name == "origin/HEAD" {
            continue;
        }
        let full_name = branch.get().name().unwrap_or(&name).to_string();
        let target = branch.get().target();
        let mut upstream = None;
        let mut ahead = 0;
        let mut behind = 0;

        if branch_type == BranchType::Local {
            if let Ok(upstream_branch) = branch.upstream() {
                upstream = upstream_branch.name().ok().flatten().map(ToOwned::to_owned);
                if let (Some(local_oid), Some(upstream_oid)) =
                    (target, upstream_branch.get().target())
                {
                    if let Ok((a, b)) = repo.graph_ahead_behind(local_oid, upstream_oid) {
                        ahead = a;
                        behind = b;
                    }
                }
            }
        }

        output.push(BranchInfo {
            name,
            full_name,
            branch_type: match branch_type {
                BranchType::Local => "local",
                BranchType::Remote => "remote",
            }
            .to_string(),
            current: branch.is_head(),
            upstream,
            target: target.map(|oid| oid.to_string()),
            ahead,
            behind,
        });
    }
    Ok(())
}

fn branch_type_order(value: &str) -> u8 {
    match value {
        "local" => 0,
        "remote" => 1,
        _ => 2,
    }
}

fn collect_project_entries(
    root: &Path,
    directory: &Path,
    depth: usize,
    max_entries: usize,
    entries: &mut Vec<ProjectFileEntry>,
) -> Result<(), GitboxError> {
    let mut directories = VecDeque::from([(directory.to_path_buf(), depth)]);
    while let Some((current_directory, current_depth)) = directories.pop_front() {
        if entries.len() >= max_entries {
            break;
        }

        let mut children = fs::read_dir(&current_directory)?
            .filter_map(Result::ok)
            .filter(|entry| {
                let name = entry.file_name();
                !is_skipped_project_entry(name.to_string_lossy().as_ref())
            })
            .collect::<Vec<_>>();

        children.sort_by(|left, right| {
            let left_type = left.file_type().ok();
            let right_type = right.file_type().ok();
            let left_is_dir = left_type
                .as_ref()
                .map(|kind| kind.is_dir())
                .unwrap_or(false);
            let right_is_dir = right_type
                .as_ref()
                .map(|kind| kind.is_dir())
                .unwrap_or(false);

            right_is_dir.cmp(&left_is_dir).then_with(|| {
                left.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&right.file_name().to_string_lossy().to_lowercase())
            })
        });

        for child in children {
            if entries.len() >= max_entries {
                break;
            }

            let file_type = child.file_type()?;
            let metadata = child.metadata()?;
            let child_path = child.path();
            let relative = child_path
                .strip_prefix(root)
                .map_err(|_| GitboxError::Message("无法读取项目文件路径".to_string()))?;
            let parent = relative
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(repo_path_string);
            let directory = file_type.is_dir();

            entries.push(ProjectFileEntry {
                path: repo_path_string(relative),
                name: child.file_name().to_string_lossy().to_string(),
                parent,
                depth: current_depth,
                directory,
                size: if directory {
                    None
                } else {
                    Some(metadata.len())
                },
            });

            if directory {
                directories.push_back((child_path, current_depth + 1));
            }
        }
    }

    Ok(())
}

fn is_skipped_project_entry(name: &str) -> bool {
    matches!(name, ".git" | "node_modules" | "target")
}

fn project_workdir_root(path: &str) -> Result<PathBuf, GitboxError> {
    let repo = Repository::discover(path)?;
    fs::canonicalize(repo_workdir(&repo)?).map_err(Into::into)
}

fn project_file_mutation(
    root: &Path,
    target: &Path,
    directory: bool,
    message: String,
) -> Result<ProjectFileMutation, GitboxError> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| GitboxError::Message("无法读取项目文件路径".to_string()))?;
    Ok(ProjectFileMutation {
        path: repo_path_string(relative),
        directory,
        message,
    })
}

fn clean_optional_project_relative_path(
    value: Option<String>,
) -> Result<Option<PathBuf>, GitboxError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "__gitbox_project_root__" {
        return Ok(None);
    }
    clean_project_relative_path(trimmed.to_string()).map(Some)
}

fn resolve_project_directory(
    root: &Path,
    directory_path: Option<String>,
) -> Result<PathBuf, GitboxError> {
    let Some(relative) = clean_optional_project_relative_path(directory_path)? else {
        return Ok(root.to_path_buf());
    };
    let target = fs::canonicalize(root.join(relative))?;
    if !target.starts_with(root) {
        return Err(GitboxError::Message("文件夹不在当前项目内".to_string()));
    }
    if !fs::metadata(&target)?.is_dir() {
        return Err(GitboxError::Message("目标不是文件夹".to_string()));
    }
    Ok(target)
}

fn resolve_project_entry(root: &Path, file_path: String) -> Result<PathBuf, GitboxError> {
    let relative = clean_project_relative_path(file_path)?;
    let target = fs::canonicalize(root.join(relative))?;
    if !target.starts_with(root) {
        return Err(GitboxError::Message("文件不在当前项目内".to_string()));
    }
    Ok(target)
}

fn clean_project_entry_name(value: String) -> Result<String, GitboxError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GitboxError::Message("请输入名称".to_string()));
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        return Err(GitboxError::Message("名称不能包含路径分隔符".to_string()));
    }

    let path = PathBuf::from(trimmed);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(GitboxError::Message("名称无效".to_string()));
    }
    Ok(trimmed.to_string())
}

fn clean_project_child_path(value: String, empty_message: &str) -> Result<PathBuf, GitboxError> {
    let trimmed = clean_ref_input(value, empty_message)?;
    if trimmed.contains('\\') {
        return Err(GitboxError::Message("名称不能包含反斜杠".to_string()));
    }
    if trimmed
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(GitboxError::Message(
            "名称不能包含空路径、. 或 ..".to_string(),
        ));
    }

    let path = PathBuf::from(trimmed);
    let invalid = path.components().any(|component| {
        matches!(
            component,
            Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });

    if path.is_absolute() || invalid {
        return Err(GitboxError::Message(
            "名称不能包含空路径、. 或 ..".to_string(),
        ));
    }

    Ok(path)
}

fn unique_project_copy_path(target: &Path, directory: bool) -> PathBuf {
    if !target.exists() {
        return target.to_path_buf();
    }

    let parent = target.parent().unwrap_or_else(|| Path::new(""));
    let file_name = target
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "副本".to_string());
    let (stem, extension) = if directory {
        (file_name, None)
    } else {
        (
            target
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| file_name.clone()),
            target
                .extension()
                .map(|value| value.to_string_lossy().to_string()),
        )
    };

    for index in 1..1000 {
        let suffix = if index == 1 {
            " 副本".to_string()
        } else {
            format!(" 副本 {index}")
        };
        let candidate_name = match extension.as_deref() {
            Some(extension) if !extension.is_empty() => format!("{stem}{suffix}.{extension}"),
            _ => format!("{stem}{suffix}"),
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    target.to_path_buf()
}

fn copy_project_entry_recursive(source: &Path, target: &Path) -> Result<(), GitboxError> {
    let metadata = fs::metadata(source)?;
    if metadata.is_dir() {
        fs::create_dir(target)?;
        for child in fs::read_dir(source)? {
            let child = child?;
            copy_project_entry_recursive(&child.path(), &target.join(child.file_name()))?;
        }
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target)?;
    }
    Ok(())
}

fn clean_project_relative_path(value: String) -> Result<PathBuf, GitboxError> {
    let trimmed = clean_ref_input(value, "请选择项目文件")?;
    let path = PathBuf::from(trimmed);
    let invalid = path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });

    if path.is_absolute() || invalid {
        return Err(GitboxError::Message("项目文件路径无效".to_string()));
    }

    Ok(path)
}

fn clean_ref_input(value: String, empty_message: &str) -> Result<String, GitboxError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GitboxError::Message(empty_message.to_string()));
    }
    Ok(trimmed.to_string())
}

fn clean_optional_arg(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "ALL")
}

fn clean_optional_args(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut cleaned = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || value == "ALL" || cleaned.iter().any(|item| item == value) {
            continue;
        }
        cleaned.push(value.to_string());
    }
    cleaned
}

fn parent_or_empty_tree(repo: &Repository, oid: &str) -> Result<String, GitboxError> {
    let object = repo.revparse_single(oid)?;
    let commit = object.peel_to_commit()?;
    if commit.parent_count() > 0 {
        Ok(format!("{oid}^"))
    } else {
        Ok("4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string())
    }
}

fn commit_matches_author(commit: &git2::Commit<'_>, author: Option<&str>) -> bool {
    let Some(author) = author else {
        return true;
    };
    let needle = author.to_lowercase();
    let signature = commit.author();
    let name = signature.name().unwrap_or_default();
    let email = signature.email().unwrap_or_default();
    let display = if email.is_empty() {
        name.to_string()
    } else {
        format!("{name} <{email}>")
    };
    [name, email, display.as_str()]
        .iter()
        .any(|value| value.to_lowercase().contains(&needle))
}

fn commit_matches_any_author(commit: &git2::Commit<'_>, authors: &[String]) -> bool {
    authors.is_empty()
        || authors
            .iter()
            .any(|author| commit_matches_author(commit, Some(author.as_str())))
}

fn commit_matches_query(commit: &git2::Commit<'_>, refs: &[String], query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };
    let needle = query.to_lowercase();
    let author = commit.author();
    let haystacks = [
        commit.id().to_string(),
        commit.summary().unwrap_or_default().to_string(),
        commit.body().unwrap_or_default().to_string(),
        author.name().unwrap_or_default().to_string(),
        author.email().unwrap_or_default().to_string(),
        refs.join(" "),
    ];
    haystacks
        .iter()
        .any(|value| value.to_lowercase().contains(&needle))
}

fn local_branch_from_remote(remote_branch: &str) -> String {
    remote_branch
        .split_once('/')
        .map(|(_, branch)| branch)
        .filter(|branch| !branch.trim().is_empty())
        .unwrap_or(remote_branch)
        .to_string()
}

fn split_remote_branch(remote_branch: &str) -> Result<(String, String), GitboxError> {
    let (remote, branch) = remote_branch
        .split_once('/')
        .ok_or_else(|| GitboxError::Message("远程分支名称应为“远程/分支”格式".to_string()))?;
    if remote.trim().is_empty() || branch.trim().is_empty() || branch == "HEAD" {
        return Err(GitboxError::Message("请选择有效的远程分支".to_string()));
    }
    Ok((remote.to_string(), branch.to_string()))
}

fn short_ref(value: &str) -> String {
    value.chars().take(10).collect()
}

fn operation_label(value: &str) -> &'static str {
    match value {
        "merge" => "合并",
        "rebase" => "变基",
        "cherry-pick" => "挑选提交",
        "revert" => "反向提交",
        _ => "Git 操作",
    }
}

fn operation_action_label(value: &str) -> &'static str {
    match value {
        "continue" => "继续",
        "abort" => "终止",
        "skip" => "跳过",
        _ => "未知",
    }
}

fn conflict_side_label(value: &str) -> &'static str {
    match value {
        "--ours" | "ours" => "当前版本",
        "--theirs" | "theirs" => "传入版本",
        "base" => "基线版本",
        _ => "所选版本",
    }
}

fn stash_action_label(value: &str) -> &'static str {
    match value {
        "apply" => "应用",
        "pop" => "弹出",
        "drop" => "删除",
        _ => "处理",
    }
}

fn format_ref_label(value: &str) -> String {
    value
        .trim_start_matches("refs/heads/")
        .trim_start_matches("refs/remotes/")
        .trim_start_matches("refs/tags/")
        .to_string()
}

fn conflict_side_flag(side: &str) -> Result<&'static str, GitboxError> {
    match side {
        "ours" => Ok("--ours"),
        "theirs" => Ok("--theirs"),
        _ => Err(GitboxError::Message(
            "冲突处理方式只支持当前或传入".to_string(),
        )),
    }
}

fn read_index_stage(
    workdir: &Path,
    stage: usize,
    pathspec: &str,
) -> Result<Option<String>, GitboxError> {
    let output = run_git_raw(
        workdir,
        vec!["show".to_string(), format!(":{stage}:{pathspec}")],
        None,
    )?;
    if output.success {
        Ok(Some(output.stdout))
    } else {
        Ok(None)
    }
}

fn split_preserving_newlines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }

    let mut lines = content
        .split_inclusive('\n')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if !content.ends_with('\n') {
        let consumed = lines.iter().map(String::len).sum::<usize>();
        if consumed < content.len() {
            lines.push(content[consumed..].to_string());
        }
    }
    lines
}

fn parse_conflict_blocks_for_response(content: &str) -> Vec<ConflictBlock> {
    let lines = split_preserving_newlines(content);
    parse_conflict_blocks(&lines)
        .into_iter()
        .map(|block| ConflictBlock {
            index: block.index,
            ours: block.ours.concat(),
            base: if block.base.is_empty() {
                None
            } else {
                Some(block.base.concat())
            },
            theirs: block.theirs.concat(),
        })
        .collect()
}

fn parse_conflict_blocks(lines: &[String]) -> Vec<ParsedConflictBlock> {
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if !lines[index].starts_with("<<<<<<< ") {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        let mut ours = Vec::new();
        let mut base = Vec::new();
        let mut theirs = Vec::new();
        let mut section = "ours";

        while index < lines.len() {
            let line = &lines[index];
            if line.starts_with("||||||| ") {
                section = "base";
                index += 1;
                continue;
            }
            if line.starts_with("=======") {
                section = "theirs";
                index += 1;
                continue;
            }
            if line.starts_with(">>>>>>> ") {
                blocks.push(ParsedConflictBlock {
                    index: blocks.len(),
                    start,
                    end: index,
                    ours,
                    base,
                    theirs,
                });
                index += 1;
                break;
            }

            match section {
                "ours" => ours.push(line.clone()),
                "base" => base.push(line.clone()),
                _ => theirs.push(line.clone()),
            }
            index += 1;
        }
    }

    blocks
}

fn repo_is_clean(repo: &Repository, include_ignored: bool) -> Result<bool, GitboxError> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .include_ignored(include_ignored);
    Ok(repo.statuses(Some(&mut opts))?.is_empty())
}

fn commit_parents(repo: &Repository) -> Result<Vec<git2::Commit<'_>>, GitboxError> {
    match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit()?;
            Ok(vec![commit])
        }
        Err(err) if err.code() == git2::ErrorCode::UnbornBranch => Ok(Vec::new()),
        Err(err) if err.code() == git2::ErrorCode::NotFound => Ok(Vec::new()),
        Err(err) => Err(err.into()),
    }
}

fn signed_off_message(message: &str, signature: &git2::Signature<'_>) -> String {
    let name = signature.name().unwrap_or("Unknown");
    let email = signature.email().unwrap_or("unknown@example.invalid");
    let signoff = format!("Signed-off-by: {name} <{email}>");
    if message.lines().any(|line| line.trim() == signoff) {
        message.to_string()
    } else {
        format!("{}\n\n{}", message.trim_end(), signoff)
    }
}

fn has_index_change(status: Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
            | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE,
    )
}

fn has_worktree_change(status: Status) -> bool {
    status.intersects(
        Status::WT_NEW
            | Status::WT_MODIFIED
            | Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE,
    )
}

fn status_kind(status: Status) -> String {
    let mut parts = Vec::new();
    if status.contains(Status::INDEX_NEW) || status.contains(Status::WT_NEW) {
        parts.push("added");
    }
    if status.contains(Status::INDEX_MODIFIED) || status.contains(Status::WT_MODIFIED) {
        parts.push("modified");
    }
    if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED) {
        parts.push("deleted");
    }
    if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED) {
        parts.push("renamed");
    }
    if status.contains(Status::INDEX_TYPECHANGE) || status.contains(Status::WT_TYPECHANGE) {
        parts.push("typechange");
    }
    if status.contains(Status::CONFLICTED) {
        parts.push("conflicted");
    }
    if status.contains(Status::IGNORED) {
        parts.push("ignored");
    }
    if parts.is_empty() {
        parts.push("unknown");
    }
    parts.join("|")
}

fn delta_new_path(delta: git2::DiffDelta<'_>) -> Option<String> {
    delta.new_file().path().map(path_string)
}

fn delta_old_path(delta: git2::DiffDelta<'_>) -> Option<String> {
    delta.old_file().path().map(path_string)
}

fn normalize_repo_path(repo: &Repository, input: &str) -> Result<PathBuf, GitboxError> {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        let workdir = repo_workdir(repo)?;
        Ok(path
            .strip_prefix(workdir)
            .map_err(|_| GitboxError::Message(format!("{input} 不在当前仓库工作区内")))?
            .to_path_buf())
    } else {
        Ok(path)
    }
}

fn repo_workdir(repo: &Repository) -> Result<PathBuf, GitboxError> {
    repo.workdir()
        .map(Path::to_path_buf)
        .ok_or_else(|| GitboxError::Message("暂不支持裸仓库".to_string()))
}

fn repo_root(repo: &Repository) -> Result<PathBuf, GitboxError> {
    if let Some(workdir) = repo.workdir() {
        Ok(workdir.to_path_buf())
    } else {
        repo.path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| GitboxError::Message("无法识别仓库根目录".to_string()))
    }
}

fn repo_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn git_args_with_paths(prefix: &[&str], paths: &[String]) -> Vec<String> {
    let mut args = prefix.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    args.push("--".to_string());
    args.extend(paths.iter().cloned());
    args
}

fn run_git(workdir: &Path, args: Vec<String>, input: Option<&str>) -> Result<String, GitboxError> {
    let output = run_git_raw(workdir, args, input)?;
    if output.success {
        Ok(output.stdout)
    } else {
        Err(GitboxError::Message(output.failure_message()))
    }
}

fn run_git_operation(
    path: &str,
    args: Vec<String>,
    success_message: String,
    conflict_message: String,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let output = run_git_raw(&workdir, args, None)?;
    if output.success {
        return Ok(CommandResult {
            ok: true,
            message: success_message,
            output: output.combined_output(),
        });
    }

    let state = operation_state_core(path)?;
    if state.active {
        Ok(CommandResult {
            ok: false,
            message: conflict_message,
            output: output.combined_output(),
        })
    } else {
        Err(GitboxError::Message(output.failure_message()))
    }
}

fn run_git_raw(
    workdir: &Path,
    args: Vec<String>,
    input: Option<&str>,
) -> Result<GitProcessOutput, GitboxError> {
    let mut command = Command::new("git");
    command
        .current_dir(workdir)
        .args(args)
        .env("GIT_EDITOR", "true")
        .env("GIT_SEQUENCE_EDITOR", "true");
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;
    if let Some(input) = input {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| GitboxError::Message("无法写入 Git 子进程标准输入".to_string()))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(GitProcessOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn parse_diff_hunks(text: &str) -> Vec<DiffHunk> {
    let mut file_header = Vec::<String>::new();
    let mut current_hunk = Vec::<String>::new();
    let mut hunks = Vec::<DiffHunk>::new();
    let mut current_header = String::new();
    let mut current_ranges = (0, 0, 0, 0);

    for line in text.split_inclusive('\n') {
        if line.starts_with("diff --git ") {
            if !current_hunk.is_empty() {
                push_hunk(
                    &mut hunks,
                    &file_header,
                    &current_header,
                    current_ranges,
                    &current_hunk,
                );
                current_hunk.clear();
            }
            file_header.clear();
            file_header.push(line.to_string());
        } else if line.starts_with("@@ ") {
            if !current_hunk.is_empty() {
                push_hunk(
                    &mut hunks,
                    &file_header,
                    &current_header,
                    current_ranges,
                    &current_hunk,
                );
                current_hunk.clear();
            }
            current_header = line.trim_end().to_string();
            current_ranges = parse_hunk_ranges(line);
            current_hunk.push(line.to_string());
        } else if current_hunk.is_empty() {
            if !file_header.is_empty() {
                file_header.push(line.to_string());
            }
        } else {
            current_hunk.push(line.to_string());
        }
    }

    if !current_hunk.is_empty() {
        push_hunk(
            &mut hunks,
            &file_header,
            &current_header,
            current_ranges,
            &current_hunk,
        );
    }

    hunks
}

fn push_hunk(
    hunks: &mut Vec<DiffHunk>,
    file_header: &[String],
    header: &str,
    ranges: (i32, i32, i32, i32),
    hunk: &[String],
) {
    let mut patch = String::new();
    for line in file_header {
        patch.push_str(line);
    }
    for line in hunk {
        patch.push_str(line);
    }
    hunks.push(DiffHunk {
        index: hunks.len(),
        header: header.to_string(),
        old_start: ranges.0,
        old_lines: ranges.1,
        new_start: ranges.2,
        new_lines: ranges.3,
        patch,
    });
}

fn parse_hunk_ranges(header: &str) -> (i32, i32, i32, i32) {
    let mut parts = header.split_whitespace();
    let _ = parts.next();
    let old = parts.next().unwrap_or("-0,0");
    let new = parts.next().unwrap_or("+0,0");
    let (old_start, old_lines) = parse_range(old, '-');
    let (new_start, new_lines) = parse_range(new, '+');
    (old_start, old_lines, new_start, new_lines)
}

fn parse_range(token: &str, prefix: char) -> (i32, i32) {
    let clean = token.trim_start_matches(prefix);
    let mut parts = clean.splitn(2, ',');
    let start = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let lines = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    (start, lines)
}

fn storage_conn(app: &AppHandle) -> Result<Connection, GitboxError> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(Connection::open(dir.join("gitbox.sqlite3"))?)
}

fn init_storage(conn: &Connection) -> Result<(), GitboxError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS recent_repositories (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            branch TEXT,
            last_opened_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS shelves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_path TEXT NOT NULL,
            message TEXT NOT NULL,
            stash_ref TEXT NOT NULL,
            stash_oid TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            applied_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_shelves_repo_path ON shelves(repo_path);
        ",
    )?;
    Ok(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Repository::init(dir.path()).expect("init repo");
        {
            let mut config = repo.config().expect("config");
            config.set_str("user.name", "gitbox").expect("user name");
            config
                .set_str("user.email", "gitbox@example.test")
                .expect("user email");
        }
        (dir, repo)
    }

    fn write_file(root: &Path, path: &str, content: &str) {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(full, content).expect("write file");
    }

    fn initial_commit(root: &Path) {
        write_file(root, "README.md", "hello\n");
        stage_paths_core(root.to_str().unwrap(), vec!["README.md".to_string()]).expect("stage");
        commit_core(root.to_str().unwrap(), "initial".to_string()).expect("commit");
    }

    #[test]
    fn project_file_listing_keeps_root_files_before_deep_descendants() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "src/main.rs", "fn main() {}\n");
        write_file(dir.path(), "README.md", "hello\n");

        let files =
            list_project_files_core(dir.path().to_str().unwrap(), Some(2)).expect("project files");
        let paths = files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["src", "README.md"]);
        assert_eq!(files[1].parent, None);
        assert!(!files[1].directory);
    }

    #[test]
    fn project_file_listing_keeps_later_directory_children_before_deep_descendants() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "aaa/deep/one.txt", "one\n");
        write_file(dir.path(), "zzz/child.txt", "child\n");

        let files =
            list_project_files_core(dir.path().to_str().unwrap(), Some(4)).expect("project files");
        let paths = files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["aaa", "zzz", "aaa/deep", "zzz/child.txt"]);
        assert_eq!(files[3].parent.as_deref(), Some("zzz"));
        assert!(!files[3].directory);
    }

    #[test]
    fn project_file_operations_create_copy_move_rename_and_delete() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();

        let docs =
            create_project_directory_core(path, None, "docs".to_string()).expect("create dir");
        assert_eq!(docs.path, "docs");
        assert!(docs.directory);

        let file =
            create_project_file_core(path, Some("docs".to_string()), "notes.txt".to_string())
                .expect("create file");
        assert_eq!(file.path, "docs/notes.txt");
        assert!(dir.path().join("docs/notes.txt").is_file());

        let nested =
            create_project_file_core(path, Some("docs".to_string()), "drafts/one.txt".to_string())
                .expect("create nested file");
        assert_eq!(nested.path, "docs/drafts/one.txt");
        assert!(dir.path().join("docs/drafts/one.txt").is_file());

        let nested_dir =
            create_project_directory_core(path, Some("docs".to_string()), "images/raw".to_string())
                .expect("create nested dir");
        assert_eq!(nested_dir.path, "docs/images/raw");
        assert!(dir.path().join("docs/images/raw").is_dir());

        let renamed =
            rename_project_entry_core(path, "docs/notes.txt".to_string(), "guide.md".to_string())
                .expect("rename file");
        assert_eq!(renamed.path, "docs/guide.md");

        let saved = save_project_file_core(
            path,
            "docs/guide.md".to_string(),
            "updated notes\n".to_string(),
        )
        .expect("save file");
        assert_eq!(saved.content.as_deref(), Some("updated notes\n"));
        assert_eq!(
            fs::read_to_string(dir.path().join("docs/guide.md")).unwrap(),
            "updated notes\n"
        );

        let copied =
            copy_project_entry_core(path, "docs/guide.md".to_string(), Some("docs".to_string()))
                .expect("copy file");
        assert_eq!(copied.path, "docs/guide 副本.md");
        assert!(dir.path().join("docs/guide 副本.md").is_file());

        create_project_directory_core(path, None, "archive".to_string()).expect("create archive");
        let moved = move_project_entry_core(
            path,
            "docs/guide.md".to_string(),
            Some("archive".to_string()),
        )
        .expect("move file");
        assert_eq!(moved.path, "archive/guide.md");
        assert!(dir.path().join("archive/guide.md").is_file());
        assert!(!dir.path().join("docs/guide.md").exists());

        delete_project_entry_core(path, "docs/guide 副本.md".to_string()).expect("delete copy");
        assert!(!dir.path().join("docs/guide 副本.md").exists());
    }

    #[test]
    fn status_stage_and_commit_lifecycle() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "src/main.rs", "fn main() {}\n");

        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.untracked, 1);

        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["src/main.rs".to_string()],
        )
        .expect("stage");

        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.staged, 1);

        let commit =
            commit_core(dir.path().to_str().unwrap(), "add main".to_string()).expect("commit");
        assert_eq!(commit.oid.len(), 40);
        assert!(
            repo_status_core(dir.path().to_str().unwrap(), false)
                .expect("status")
                .branch
                .clean
        );
    }

    #[test]
    fn amend_commit_can_replace_head_and_add_signoff() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "src/lib.rs", "pub fn added() {}\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["src/lib.rs".to_string()])
            .expect("stage");

        let amended = commit_with_options_core(
            dir.path().to_str().unwrap(),
            "initial amended".to_string(),
            true,
            true,
        )
        .expect("amend");

        let commits = list_commits_core(dir.path().to_str().unwrap(), Some(10)).expect("log");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].oid, amended.oid);
        assert_eq!(commits[0].summary, "initial amended");
        assert!(commits[0]
            .body
            .contains("Signed-off-by: gitbox <gitbox@example.test>"));
        assert_eq!(
            fs::read_to_string(dir.path().join("src/lib.rs")).unwrap(),
            "pub fn added() {}\n"
        );
    }

    #[test]
    fn commit_can_override_author_with_system_git() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "author.txt", "author override\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["author.txt".to_string()])
            .expect("stage");

        let result = commit_with_full_options_core(
            dir.path().to_str().unwrap(),
            "author override".to_string(),
            false,
            false,
            false,
            Some("Ada Lovelace <ada@example.test>".to_string()),
        )
        .expect("commit with author");
        let details =
            commit_details_core(dir.path().to_str().unwrap(), result.oid).expect("details");
        assert_eq!(details.commit.author_name, "Ada Lovelace");
        assert_eq!(details.commit.author_email, "ada@example.test");
    }

    #[test]
    fn commit_log_can_filter_all_refs_by_query_author_and_path() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/search".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(dir.path(), "src/search.rs", "pub fn search() {}\n");
        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["src/search.rs".to_string()],
        )
        .expect("stage");
        commit_with_full_options_core(
            dir.path().to_str().unwrap(),
            "search branch work".to_string(),
            false,
            false,
            false,
            Some("Grace Hopper <grace@example.test>".to_string()),
        )
        .expect("commit");
        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");

        let commits = list_commits_filtered_core(
            dir.path().to_str().unwrap(),
            Some(20),
            None,
            Some("search branch".to_string()),
            Some("grace".to_string()),
            Some("src/search.rs".to_string()),
        )
        .expect("filtered commits");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].summary, "search branch work");
        assert!(commits[0]
            .refs
            .iter()
            .any(|reference| reference == "feature/search"));
    }

    #[test]
    fn commit_log_can_filter_multiple_authors_and_paths() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        write_file(dir.path(), "src/one.rs", "pub fn one() {}\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["src/one.rs".to_string()])
            .expect("stage one");
        commit_with_full_options_core(
            dir.path().to_str().unwrap(),
            "ada path work".to_string(),
            false,
            false,
            false,
            Some("Ada Lovelace <ada@example.test>".to_string()),
        )
        .expect("commit ada");

        write_file(dir.path(), "docs/two.md", "two\n");
        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["docs/two.md".to_string()],
        )
        .expect("stage two");
        commit_with_full_options_core(
            dir.path().to_str().unwrap(),
            "grace docs work".to_string(),
            false,
            false,
            false,
            Some("Grace Hopper <grace@example.test>".to_string()),
        )
        .expect("commit grace");

        write_file(dir.path(), "README.md", "ignored\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage readme");
        commit_with_full_options_core(
            dir.path().to_str().unwrap(),
            "other work".to_string(),
            false,
            false,
            false,
            Some("Linus Torvalds <linus@example.test>".to_string()),
        )
        .expect("commit other");

        let commits = list_commits_filtered_multi_core(
            dir.path().to_str().unwrap(),
            Some(20),
            None,
            None,
            vec![
                "Ada Lovelace <ada@example.test>".to_string(),
                "Grace Hopper <grace@example.test>".to_string(),
            ],
            vec!["src/one.rs".to_string(), "docs/two.md".to_string()],
        )
        .expect("multi filtered commits");
        let summaries = commits
            .iter()
            .map(|commit| commit.summary.as_str())
            .collect::<Vec<_>>();
        assert_eq!(summaries.len(), 2);
        assert!(summaries.contains(&"ada path work"));
        assert!(summaries.contains(&"grace docs work"));
    }

    #[test]
    fn remote_management_lifecycle() {
        let (dir, repo) = test_repo();
        initial_commit(dir.path());

        add_remote_core(
            dir.path().to_str().unwrap(),
            "origin".to_string(),
            "https://example.test/project.git".to_string(),
        )
        .expect("add remote");
        let remotes = remote_infos(&repo).expect("remote infos");
        assert_eq!(remotes.len(), 1);
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(
            remotes[0].url.as_deref(),
            Some("https://example.test/project.git")
        );

        update_remote_core(
            dir.path().to_str().unwrap(),
            "origin".to_string(),
            "https://example.test/project-renamed.git".to_string(),
            Some("ssh://example.test/project-renamed.git".to_string()),
        )
        .expect("update remote");
        let remotes = remote_infos(&repo).expect("remote infos after update");
        assert_eq!(
            remotes[0].url.as_deref(),
            Some("https://example.test/project-renamed.git")
        );
        assert_eq!(
            remotes[0].push_url.as_deref(),
            Some("ssh://example.test/project-renamed.git")
        );

        delete_remote_core(dir.path().to_str().unwrap(), "origin".to_string())
            .expect("delete remote");
        let remotes = remote_infos(&repo).expect("remote infos after delete");
        assert!(remotes.is_empty());
    }

    #[test]
    fn push_with_options_sets_upstream_and_pushes_tags() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        let remote_repo = Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        run_git(
            dir.path(),
            vec!["tag".to_string(), "v1.0".to_string()],
            None,
        )
        .expect("create tag");

        let result = push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            true,
            false,
            true,
        )
        .expect("push");
        assert!(result.ok);

        let upstream = run_git(
            dir.path(),
            vec![
                "rev-parse".to_string(),
                "--abbrev-ref".to_string(),
                "--symbolic-full-name".to_string(),
                "@{u}".to_string(),
            ],
            None,
        )
        .expect("upstream");
        assert_eq!(upstream.trim(), format!("origin/{branch}"));
        assert!(remote_repo
            .find_reference(&format!("refs/heads/{branch}"))
            .is_ok());
        assert!(remote_repo.find_reference("refs/tags/v1.0").is_ok());
    }

    #[test]
    fn checkout_remote_branch_creates_tracking_local_branch() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        create_branch_core(path, "feature/remote".to_string(), true, None)
            .expect("create remote feature");
        write_file(dir.path(), "feature.txt", "remote branch\n");
        stage_paths_core(path, vec!["feature.txt".to_string()]).expect("stage feature");
        commit_core(path, "remote feature".to_string()).expect("commit feature");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some("feature/remote".to_string()),
            false,
            false,
            false,
        )
        .expect("push feature");
        checkout_branch_core(path, "master".to_string()).expect("checkout master");
        delete_branch_core(path, "feature/remote".to_string(), true).expect("delete local");
        fetch_core(path, Some("origin".to_string()), false).expect("fetch remote");

        checkout_remote_branch_core(path, "origin/feature/remote".to_string(), None)
            .expect("checkout remote tracking");
        let branches = list_branches_core(path).expect("branches");
        let local = branches
            .branches
            .iter()
            .find(|branch| branch.name == "feature/remote")
            .expect("local tracking branch");
        assert!(local.current);
        assert_eq!(local.upstream.as_deref(), Some("origin/feature/remote"));

        set_branch_upstream_core(path, "feature/remote".to_string(), None).expect("unset upstream");
        let branches = list_branches_core(path).expect("branches after unset");
        let local = branches
            .branches
            .iter()
            .find(|branch| branch.name == "feature/remote")
            .expect("local branch after unset");
        assert!(local.upstream.is_none());

        set_branch_upstream_core(
            path,
            "feature/remote".to_string(),
            Some("origin/feature/remote".to_string()),
        )
        .expect("set upstream");
        let branches = list_branches_core(path).expect("branches after set");
        let local = branches
            .branches
            .iter()
            .find(|branch| branch.name == "feature/remote")
            .expect("local branch after set");
        assert_eq!(local.upstream.as_deref(), Some("origin/feature/remote"));
    }

    #[test]
    fn fetch_prune_removes_deleted_remote_tracking_branch() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        let remote_repo = Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        create_branch_core(path, "stale".to_string(), true, None).expect("create stale");
        write_file(dir.path(), "stale.txt", "stale branch\n");
        stage_paths_core(path, vec!["stale.txt".to_string()]).expect("stage stale");
        commit_core(path, "stale branch".to_string()).expect("commit stale");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some("stale".to_string()),
            false,
            false,
            false,
        )
        .expect("push stale");
        checkout_branch_core(path, "master".to_string()).expect("checkout master");
        delete_branch_core(path, "stale".to_string(), true).expect("delete local stale");
        fetch_core(path, Some("origin".to_string()), false).expect("fetch stale");
        assert!(list_branches_core(path)
            .expect("branches with stale")
            .branches
            .iter()
            .any(|branch| branch.name == "origin/stale"));

        let mut stale_ref = remote_repo
            .find_reference("refs/heads/stale")
            .expect("remote stale ref");
        stale_ref.delete().expect("delete remote ref");
        fetch_core(path, Some("origin".to_string()), true).expect("fetch prune");
        assert!(!list_branches_core(path)
            .expect("branches after prune")
            .branches
            .iter()
            .any(|branch| branch.name == "origin/stale"));
    }

    #[test]
    fn delete_remote_branch_removes_remote_ref_and_tracking_branch() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        let remote_repo = Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        create_branch_core(path, "delete-me".to_string(), true, None).expect("create branch");
        write_file(dir.path(), "delete-me.txt", "delete remote branch\n");
        stage_paths_core(path, vec!["delete-me.txt".to_string()]).expect("stage");
        commit_core(path, "delete remote branch".to_string()).expect("commit");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some("delete-me".to_string()),
            false,
            false,
            false,
        )
        .expect("push branch");
        checkout_branch_core(path, "master".to_string()).expect("checkout master");
        delete_branch_core(path, "delete-me".to_string(), true).expect("delete local");
        fetch_core(path, Some("origin".to_string()), false).expect("fetch remote");

        delete_remote_branch_core(path, "origin/delete-me".to_string())
            .expect("delete remote branch");
        assert!(remote_repo.find_reference("refs/heads/delete-me").is_err());
        assert!(!list_branches_core(path)
            .expect("branches after remote delete")
            .branches
            .iter()
            .any(|branch| branch.name == "origin/delete-me"));
    }

    #[test]
    fn tag_lifecycle_can_create_push_and_delete_local_and_remote() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        let remote_repo = Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        create_tag_core(
            path,
            "v1.1.0".to_string(),
            None,
            true,
            Some("release v1.1.0".to_string()),
        )
        .expect("create tag");
        assert!(list_branches_core(path)
            .expect("tags")
            .tags
            .iter()
            .any(|tag| tag.name == "v1.1.0"));

        push_tag_core(path, Some("origin".to_string()), "v1.1.0".to_string()).expect("push tag");
        assert!(remote_repo.find_reference("refs/tags/v1.1.0").is_ok());

        delete_remote_tag_core(path, Some("origin".to_string()), "v1.1.0".to_string())
            .expect("delete remote tag");
        assert!(remote_repo.find_reference("refs/tags/v1.1.0").is_err());

        delete_tag_core(path, "v1.1.0".to_string()).expect("delete local tag");
        assert!(!list_branches_core(path)
            .expect("tags after delete")
            .tags
            .iter()
            .any(|tag| tag.name == "v1.1.0"));
    }

    #[test]
    fn hunk_patch_can_be_staged() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "hello\nworld\n");

        let diff = get_diff_core(
            dir.path().to_str().unwrap(),
            Some("README.md".to_string()),
            false,
        )
        .expect("diff");
        assert_eq!(diff.hunks.len(), 1);

        stage_hunks_core(
            dir.path().to_str().unwrap(),
            vec![diff.hunks[0].patch.clone()],
            "stage".to_string(),
        )
        .expect("stage hunk");

        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.staged, 1);
    }

    #[test]
    fn hunk_patch_can_be_discarded() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "hello\nworld\n");

        let diff = get_diff_core(
            dir.path().to_str().unwrap(),
            Some("README.md".to_string()),
            false,
        )
        .expect("diff");
        assert_eq!(diff.hunks.len(), 1);

        stage_hunks_core(
            dir.path().to_str().unwrap(),
            vec![diff.hunks[0].patch.clone()],
            "discard".to_string(),
        )
        .expect("discard hunk");

        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.unstaged, 0);
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "hello\n"
        );
    }

    #[test]
    fn discard_restores_tracked_file() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "changed\n");

        discard_changes_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("discard");

        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "hello\n"
        );
    }

    #[test]
    fn shelve_and_unshelve_round_trip() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "changed\n");

        let shelf = shelve_changes_core(
            dir.path().to_str().unwrap(),
            vec!["README.md".to_string()],
            Some("work in progress".to_string()),
        )
        .expect("shelve");
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "hello\n"
        );

        unshelve_core(dir.path().to_str().unwrap(), shelf.stash_ref).expect("unshelve");
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "changed\n"
        );
    }

    #[test]
    fn commit_history_and_details_are_available() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "hello\nhistory\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage");
        let commit =
            commit_core(dir.path().to_str().unwrap(), "update readme".to_string()).expect("commit");

        let commits = list_commits_core(dir.path().to_str().unwrap(), Some(20)).expect("log");
        assert_eq!(commits[0].oid, commit.oid);
        assert_eq!(commits[0].summary, "update readme");

        let details =
            commit_details_core(dir.path().to_str().unwrap(), commit.oid).expect("details");
        assert!(details.diff.contains("+history"));
        assert_eq!(details.files[0].path, "README.md");
    }

    #[test]
    fn branch_list_create_checkout_and_delete() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/log".to_string(),
            true,
            None,
        )
        .expect("create branch");

        let branches = list_branches_core(dir.path().to_str().unwrap()).expect("branches");
        assert_eq!(branches.current.as_deref(), Some("feature/log"));
        assert!(branches
            .branches
            .iter()
            .any(|branch| branch.name == "feature/log" && branch.current));

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        delete_branch_core(
            dir.path().to_str().unwrap(),
            "feature/log".to_string(),
            false,
        )
        .expect("delete branch");

        let branches = list_branches_core(dir.path().to_str().unwrap()).expect("branches");
        assert!(!branches
            .branches
            .iter()
            .any(|branch| branch.name == "feature/log"));
    }

    #[test]
    fn cherry_pick_commit_applies_commit_to_current_branch() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/pick".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(dir.path(), "feature.txt", "picked\n");
        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["feature.txt".to_string()],
        )
        .expect("stage feature");
        let picked = commit_core(dir.path().to_str().unwrap(), "feature commit".to_string())
            .expect("commit");

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        let result =
            cherry_pick_commit_core(dir.path().to_str().unwrap(), picked.oid).expect("cherry-pick");
        assert!(result.ok);
        assert_eq!(
            fs::read_to_string(dir.path().join("feature.txt")).unwrap(),
            "picked\n"
        );
    }

    #[test]
    fn revert_commit_creates_inverse_commit() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "revert-me.txt", "temporary\n");
        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["revert-me.txt".to_string()],
        )
        .expect("stage");
        let added = commit_core(
            dir.path().to_str().unwrap(),
            "add temporary file".to_string(),
        )
        .expect("commit");

        let result =
            revert_commit_core(dir.path().to_str().unwrap(), added.oid, false).expect("revert");
        assert!(result.ok);
        assert!(!dir.path().join("revert-me.txt").exists());
        assert!(
            repo_status_core(dir.path().to_str().unwrap(), false)
                .expect("status")
                .branch
                .clean
        );
    }

    #[test]
    fn reset_hard_moves_head_and_worktree_to_selected_commit() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "two\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage two");
        let second =
            commit_core(dir.path().to_str().unwrap(), "second".to_string()).expect("commit two");

        write_file(dir.path(), "README.md", "three\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage three");
        commit_core(dir.path().to_str().unwrap(), "third".to_string()).expect("commit three");

        let result = reset_to_commit_core(
            dir.path().to_str().unwrap(),
            second.oid.clone(),
            "hard".to_string(),
        )
        .expect("reset hard");
        assert!(result.ok);
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "two\n"
        );
        assert_eq!(
            branch_summary_core(dir.path().to_str().unwrap(), false)
                .expect("summary")
                .head
                .as_deref(),
            Some(second.oid.as_str())
        );
    }

    #[test]
    fn undo_last_commit_moves_changes_back_to_index() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "undo.txt", "undo me\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["undo.txt".to_string()])
            .expect("stage undo");
        let second =
            commit_core(dir.path().to_str().unwrap(), "undo target".to_string()).expect("commit");

        undo_last_commit_core(dir.path().to_str().unwrap(), true).expect("undo last commit");

        let commits = list_commits_core(dir.path().to_str().unwrap(), Some(10)).expect("log");
        assert!(!commits.iter().any(|commit| commit.oid == second.oid));
        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.staged, 1);
        assert_eq!(
            fs::read_to_string(dir.path().join("undo.txt")).unwrap(),
            "undo me\n"
        );
    }

    #[test]
    fn rebase_branch_replays_feature_on_target() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/rebase".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(dir.path(), "feature.txt", "feature\n");
        stage_paths_core(
            dir.path().to_str().unwrap(),
            vec!["feature.txt".to_string()],
        )
        .expect("stage feature");
        commit_core(dir.path().to_str().unwrap(), "feature commit".to_string()).expect("commit");

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        write_file(dir.path(), "main.txt", "main\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["main.txt".to_string()])
            .expect("stage main");
        commit_core(dir.path().to_str().unwrap(), "main commit".to_string()).expect("commit");

        checkout_branch_core(dir.path().to_str().unwrap(), "feature/rebase".to_string())
            .expect("checkout feature");
        let result = rebase_branch_core(dir.path().to_str().unwrap(), "master".to_string(), false)
            .expect("rebase");
        assert!(result.ok);
        assert_eq!(
            fs::read_to_string(dir.path().join("main.txt")).unwrap(),
            "main\n"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("feature.txt")).unwrap(),
            "feature\n"
        );
    }

    #[test]
    fn merge_conflict_can_accept_ours_and_continue() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/conflict".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(dir.path(), "README.md", "feature\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage feature");
        commit_core(dir.path().to_str().unwrap(), "feature change".to_string()).expect("commit");

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        write_file(dir.path(), "README.md", "master\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage master");
        commit_core(dir.path().to_str().unwrap(), "master change".to_string()).expect("commit");

        let merge = merge_branch_core(
            dir.path().to_str().unwrap(),
            "feature/conflict".to_string(),
            false,
            false,
            false,
        )
        .expect("merge conflict");
        assert!(!merge.ok);

        let state = operation_state_core(dir.path().to_str().unwrap()).expect("state");
        assert_eq!(state.operation.as_deref(), Some("merge"));
        assert_eq!(state.conflicted_paths, vec!["README.md".to_string()]);

        let details = conflict_details_core(dir.path().to_str().unwrap(), "README.md".to_string())
            .expect("conflict details");
        assert_eq!(details.blocks.len(), 1);

        resolve_conflict_file_core(
            dir.path().to_str().unwrap(),
            "README.md".to_string(),
            "ours".to_string(),
        )
        .expect("resolve ours");
        operation_control_core(dir.path().to_str().unwrap(), "continue".to_string())
            .expect("merge continue");

        let state = operation_state_core(dir.path().to_str().unwrap()).expect("state");
        assert!(!state.active);
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "master\n"
        );
    }

    #[test]
    fn merge_conflict_result_can_be_saved_and_marked_resolved() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/manual-merge".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(dir.path(), "README.md", "feature\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage feature");
        commit_core(dir.path().to_str().unwrap(), "feature change".to_string()).expect("commit");

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        write_file(dir.path(), "README.md", "master\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage master");
        commit_core(dir.path().to_str().unwrap(), "master change".to_string()).expect("commit");

        merge_branch_core(
            dir.path().to_str().unwrap(),
            "feature/manual-merge".to_string(),
            false,
            false,
            false,
        )
        .expect("merge conflict");

        let rejected = save_conflict_result_core(
            dir.path().to_str().unwrap(),
            "README.md".to_string(),
            "<<<<<<< HEAD\nmanual\n=======\nfeature\n>>>>>>> branch\n".to_string(),
            true,
        );
        assert!(rejected.is_err());

        save_conflict_result_core(
            dir.path().to_str().unwrap(),
            "README.md".to_string(),
            "manual result\n".to_string(),
            true,
        )
        .expect("save result");
        operation_control_core(dir.path().to_str().unwrap(), "continue".to_string())
            .expect("merge continue");

        let state = operation_state_core(dir.path().to_str().unwrap()).expect("state");
        assert!(!state.active);
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "manual result\n"
        );
    }

    #[test]
    fn file_history_blame_patch_and_ref_compare_are_available() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        write_file(dir.path(), "README.md", "hello\nworld\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage readme");
        commit_core(dir.path().to_str().unwrap(), "expand readme".to_string()).expect("commit");

        let history = file_history_core(
            dir.path().to_str().unwrap(),
            "README.md".to_string(),
            Some(10),
        )
        .expect("file history");
        assert!(history.iter().any(|entry| entry.summary == "expand readme"));

        let blame =
            blame_file_core(dir.path().to_str().unwrap(), "README.md".to_string()).expect("blame");
        assert_eq!(blame.len(), 2);
        assert!(blame.iter().any(|line| line.content == "world"));

        write_file(dir.path(), "extra.txt", "patch me\n");
        let patch = create_patch_core(
            dir.path().to_str().unwrap(),
            vec!["extra.txt".to_string()],
            false,
        )
        .expect("create patch");
        assert!(patch.output.contains("extra.txt"));
        discard_changes_core(dir.path().to_str().unwrap(), vec!["extra.txt".to_string()])
            .expect("discard extra");
        apply_patch_core(dir.path().to_str().unwrap(), patch.output, false, true)
            .expect("apply patch");
        assert_eq!(
            fs::read_to_string(dir.path().join("extra.txt")).unwrap(),
            "patch me\n"
        );

        create_branch_core(
            dir.path().to_str().unwrap(),
            "compare-target".to_string(),
            false,
            None,
        )
        .expect("create compare branch");
        let comparison = compare_refs_core(
            dir.path().to_str().unwrap(),
            "compare-target".to_string(),
            "HEAD".to_string(),
        )
        .expect("compare refs");
        assert_eq!(comparison.left, "compare-target");
        assert_eq!(comparison.right, "HEAD");
    }

    #[test]
    fn init_clone_branch_rename_and_worktree_lifecycle() {
        let parent = tempfile::tempdir().expect("parent");
        let init_path = parent.path().join("initialized");
        let initialized = init_repository_core(
            init_path.to_string_lossy().to_string(),
            false,
            Some("main".to_string()),
        )
        .expect("init repo");
        assert_eq!(initialized.branch.as_deref(), Some("main"));

        let (source_dir, _repo) = test_repo();
        initial_commit(source_dir.path());
        let clone_path = parent.path().join("clone");
        let cloned = clone_repository_core(
            source_dir.path().to_string_lossy().to_string(),
            clone_path.to_string_lossy().to_string(),
            None,
        )
        .expect("clone repo");
        assert!(Path::new(&cloned.path).join("README.md").exists());

        rename_branch_core(
            &cloned.path,
            "master".to_string(),
            "main-renamed".to_string(),
        )
        .expect("rename branch");
        let branches = list_branches_core(&cloned.path).expect("branches");
        assert!(branches
            .branches
            .iter()
            .any(|branch| branch.name == "main-renamed"));

        let worktree_path = parent.path().join("linked-worktree");
        create_worktree_core(
            &cloned.path,
            worktree_path.to_string_lossy().to_string(),
            Some("worktree-branch".to_string()),
            None,
            false,
        )
        .expect("create worktree");
        let worktrees = list_worktrees_core(&cloned.path).expect("worktrees");
        assert!(worktrees
            .iter()
            .any(|item| item.path.ends_with("linked-worktree")));
        remove_worktree_core(
            &cloned.path,
            worktree_path.to_string_lossy().to_string(),
            true,
        )
        .expect("remove worktree");
    }
}
