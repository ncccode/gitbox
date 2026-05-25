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
    pub old_text: Option<String>,
    pub new_text: Option<String>,
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
pub struct PullPreflight {
    pub remote: String,
    pub target: String,
    pub current_branch: Option<String>,
    pub head: Option<String>,
    pub target_oid: Option<String>,
    pub up_to_date: bool,
    pub fast_forward: bool,
    pub diverged: bool,
    pub local_changed_paths: Vec<String>,
    pub remote_changed_paths: Vec<String>,
    pub overlapping_paths: Vec<String>,
    pub needs_confirmation: bool,
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
    pub current_side: String,
    pub incoming_side: String,
    pub conflict_source: Option<String>,
    pub blocks: Vec<ConflictBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictBlockAnalysis {
    pub index: usize,
    pub kind: String,
    pub confidence: String,
    pub score: u8,
    pub suggested_side: Option<String>,
    pub explanation: String,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictAnalysis {
    pub path: String,
    pub blocks: Vec<ConflictBlockAnalysis>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MergePreviewSummary {
    pub clean: usize,
    pub auto_resolvable: usize,
    pub manual: usize,
    pub add_delete: usize,
    pub binary: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePreviewFile {
    pub path: String,
    pub category: String,
    pub conflict_count: usize,
    pub auto_resolvable: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePreview {
    pub target: String,
    pub head: Option<String>,
    pub target_oid: Option<String>,
    pub merge_base: Option<String>,
    pub clean: bool,
    pub files: Vec<MergePreviewFile>,
    pub summary: MergePreviewSummary,
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
        } else if let Some(message) = translate_git_failure_message(&combined) {
            message
        } else {
            combined
        }
    }
}
