mod gitbox;

use gitbox::{
    add_remote_core, apply_patch_core, blame_file_core, branch_summary_core, checkout_branch_core,
    checkout_remote_branch_core, checkout_revision_core, cherry_pick_commit_core,
    cherry_pick_files_core, cleanup_merged_branches_core, clear_stashes_core,
    clone_repository_core, commit_details_core, commit_file_diff_core, commit_message_history_core,
    commit_with_full_options_and_selection_core, compare_refs_core, conflict_details_core,
    copy_project_entry_core, create_branch_core, create_patch_core, create_project_directory_core,
    create_project_file_core, create_tag_core, create_worktree_core, delete_branch_core,
    delete_project_entry_core, delete_remote_branch_core, delete_remote_core,
    delete_remote_tag_core, delete_shelf_core, delete_tag_core, discard_changes_core,
    drop_commit_core, fetch_core, file_history_core, filter_project_directories_core,
    fixup_commit_core, get_diff_core, init_repository_core, lfs_status_core, list_branches_core,
    list_commits_filtered_multi_core, list_project_files_core, list_shelves_core,
    list_stashes_core, list_submodules_core, list_worktrees_core, mark_conflict_resolved_core,
    mark_shelf_applied, merge_branch_core, move_project_entry_core, open_project_directory_core,
    open_project_terminal_core, open_repo_core, operation_control_core, operation_state_core,
    pull_core, pull_preflight_core, push_commit_core, push_tag_core, push_with_options_core,
    read_project_file_core, rebase_advanced_core, rebase_branch_core, record_recent_repo,
    record_shelf, remove_worktree_core, rename_branch_core, rename_project_entry_core,
    repo_status_core, reset_to_commit_core, resolve_conflict_block_core,
    resolve_conflict_file_core, revert_commit_core, revert_commit_files_core,
    save_conflict_result_core, save_project_file_core, set_branch_upstream_core,
    shelve_changes_core, stage_hunks_core, stage_paths_core, stash_action_core,
    undo_last_commit_core, unshallow_repository_core, unshelve_core, unstage_paths_core,
    update_remote_core, update_submodules_core, BlameLine, BranchList, BranchSummary,
    CommandResult, CommitDetails, CommitResult, CommitSummary, ConflictDetails, DiffResponse,
    FileHistoryEntry, GitOperationState, ProjectFileContent, ProjectFileEntry, ProjectFileMutation,
    PullPreflight, RefComparison, RepoStatus, RepositoryInfo, ShelfInfo, StashInfo, SubmoduleInfo,
    WorktreeInfo,
};
use tauri::AppHandle;

type CommandResponse<T> = Result<T, String>;

fn merge_optional_filters(primary: Option<String>, extra: Option<Vec<String>>) -> Vec<String> {
    let mut values = Vec::new();
    for value in primary.into_iter().chain(extra.unwrap_or_default()) {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "ALL" || values.iter().any(|item| item == trimmed) {
            continue;
        }
        values.push(trimmed.to_string());
    }
    values
}

#[tauri::command]
fn open_repo(app: AppHandle, path: String) -> CommandResponse<RepositoryInfo> {
    let info = open_repo_core(&path).map_err(|err| err.command())?;
    record_recent_repo(&app, &info).map_err(|err| err.command())?;
    Ok(info)
}

#[tauri::command]
fn filter_project_directories(paths: Vec<String>) -> CommandResponse<Vec<String>> {
    filter_project_directories_core(paths).map_err(|err| err.command())
}

#[tauri::command]
fn open_project_directory(path: String) -> CommandResponse<CommandResult> {
    open_project_directory_core(path).map_err(|err| err.command())
}

#[tauri::command]
fn open_project_terminal(path: String) -> CommandResponse<CommandResult> {
    open_project_terminal_core(path).map_err(|err| err.command())
}

#[tauri::command]
fn init_repository(
    app: AppHandle,
    path: String,
    bare: Option<bool>,
    initial_branch: Option<String>,
) -> CommandResponse<RepositoryInfo> {
    let info = init_repository_core(path, bare.unwrap_or(false), initial_branch)
        .map_err(|err| err.command())?;
    record_recent_repo(&app, &info).map_err(|err| err.command())?;
    Ok(info)
}

#[tauri::command]
fn clone_repository(
    app: AppHandle,
    url: String,
    directory: String,
    shallow_depth: Option<u32>,
) -> CommandResponse<RepositoryInfo> {
    let info = clone_repository_core(url, directory, shallow_depth).map_err(|err| err.command())?;
    record_recent_repo(&app, &info).map_err(|err| err.command())?;
    Ok(info)
}

#[tauri::command]
fn unshallow_repository(
    path: String,
    remote_name: Option<String>,
) -> CommandResponse<CommandResult> {
    unshallow_repository_core(&path, remote_name).map_err(|err| err.command())
}

#[tauri::command]
fn repo_status(path: String, include_ignored: Option<bool>) -> CommandResponse<RepoStatus> {
    repo_status_core(&path, include_ignored.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn branch_summary(path: String) -> CommandResponse<BranchSummary> {
    branch_summary_core(&path, false).map_err(|err| err.command())
}

#[tauri::command]
fn list_project_files(
    path: String,
    limit: Option<usize>,
) -> CommandResponse<Vec<ProjectFileEntry>> {
    list_project_files_core(&path, limit).map_err(|err| err.command())
}

#[tauri::command]
fn read_project_file(path: String, file_path: String) -> CommandResponse<ProjectFileContent> {
    read_project_file_core(&path, file_path).map_err(|err| err.command())
}

#[tauri::command]
fn save_project_file(
    path: String,
    file_path: String,
    content: String,
) -> CommandResponse<ProjectFileContent> {
    save_project_file_core(&path, file_path, content).map_err(|err| err.command())
}

#[tauri::command]
fn create_project_file(
    path: String,
    directory_path: Option<String>,
    name: String,
) -> CommandResponse<ProjectFileMutation> {
    create_project_file_core(&path, directory_path, name).map_err(|err| err.command())
}

#[tauri::command]
fn create_project_directory(
    path: String,
    directory_path: Option<String>,
    name: String,
) -> CommandResponse<ProjectFileMutation> {
    create_project_directory_core(&path, directory_path, name).map_err(|err| err.command())
}

#[tauri::command]
fn rename_project_entry(
    path: String,
    file_path: String,
    new_name: String,
) -> CommandResponse<ProjectFileMutation> {
    rename_project_entry_core(&path, file_path, new_name).map_err(|err| err.command())
}

#[tauri::command]
fn delete_project_entry(path: String, file_path: String) -> CommandResponse<CommandResult> {
    delete_project_entry_core(&path, file_path).map_err(|err| err.command())
}

#[tauri::command]
fn copy_project_entry(
    path: String,
    source_path: String,
    target_directory_path: Option<String>,
) -> CommandResponse<ProjectFileMutation> {
    copy_project_entry_core(&path, source_path, target_directory_path).map_err(|err| err.command())
}

#[tauri::command]
fn move_project_entry(
    path: String,
    source_path: String,
    target_directory_path: Option<String>,
) -> CommandResponse<ProjectFileMutation> {
    move_project_entry_core(&path, source_path, target_directory_path).map_err(|err| err.command())
}

#[tauri::command]
fn get_diff(
    path: String,
    file_path: Option<String>,
    staged: Option<bool>,
) -> CommandResponse<DiffResponse> {
    get_diff_core(&path, file_path, staged.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn stage_paths(path: String, paths: Vec<String>) -> CommandResponse<CommandResult> {
    stage_paths_core(&path, paths).map_err(|err| err.command())
}

#[tauri::command]
fn unstage_paths(path: String, paths: Vec<String>) -> CommandResponse<CommandResult> {
    unstage_paths_core(&path, paths).map_err(|err| err.command())
}

#[tauri::command]
fn stage_hunks(path: String, patches: Vec<String>, mode: String) -> CommandResponse<CommandResult> {
    stage_hunks_core(&path, patches, mode).map_err(|err| err.command())
}

#[tauri::command]
fn discard_changes(path: String, paths: Vec<String>) -> CommandResponse<CommandResult> {
    discard_changes_core(&path, paths).map_err(|err| err.command())
}

#[tauri::command]
fn shelve_changes(
    app: AppHandle,
    path: String,
    paths: Vec<String>,
    message: Option<String>,
) -> CommandResponse<ShelfInfo> {
    let draft = shelve_changes_core(&path, paths, message).map_err(|err| err.command())?;
    record_shelf(&app, &path, draft).map_err(|err| err.command())
}

#[tauri::command]
fn unshelve(app: AppHandle, path: String, stash_ref: String) -> CommandResponse<CommandResult> {
    let result = unshelve_core(&path, stash_ref.clone()).map_err(|err| err.command())?;
    mark_shelf_applied(&app, &stash_ref).map_err(|err| err.command())?;
    Ok(result)
}

#[tauri::command]
fn delete_shelf(
    app: AppHandle,
    path: String,
    stash_ref: String,
    drop_stash: Option<bool>,
) -> CommandResponse<CommandResult> {
    delete_shelf_core(&app, &path, stash_ref, drop_stash.unwrap_or(true))
        .map_err(|err| err.command())
}

#[tauri::command]
fn list_shelves(app: AppHandle, path: String) -> CommandResponse<Vec<ShelfInfo>> {
    list_shelves_core(&app, &path).map_err(|err| err.command())
}

#[tauri::command]
fn commit(
    path: String,
    message: String,
    amend: Option<bool>,
    sign_off: Option<bool>,
    gpg_sign: Option<bool>,
    author: Option<String>,
    include_worktree: Option<bool>,
    selected_paths: Option<Vec<String>>,
) -> CommandResponse<CommitResult> {
    commit_with_full_options_and_selection_core(
        &path,
        message,
        amend.unwrap_or(false),
        sign_off.unwrap_or(false),
        gpg_sign.unwrap_or(false),
        author,
        include_worktree.unwrap_or(false),
        selected_paths,
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn fetch(
    path: String,
    remote_name: Option<String>,
    prune: Option<bool>,
) -> CommandResponse<CommandResult> {
    fetch_core(&path, remote_name, prune.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn pull_preflight(path: String, remote_name: Option<String>) -> CommandResponse<PullPreflight> {
    pull_preflight_core(&path, remote_name).map_err(|err| err.command())
}

#[tauri::command]
fn pull(
    path: String,
    remote_name: Option<String>,
    smart_merge: Option<bool>,
) -> CommandResponse<CommandResult> {
    pull_core(&path, remote_name, smart_merge.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn push(
    path: String,
    remote_name: Option<String>,
    target_branch: Option<String>,
    set_upstream: Option<bool>,
    force_with_lease: Option<bool>,
    push_tags: Option<bool>,
) -> CommandResponse<CommandResult> {
    push_with_options_core(
        &path,
        remote_name,
        target_branch,
        set_upstream.unwrap_or(false),
        force_with_lease.unwrap_or(false),
        push_tags.unwrap_or(false),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn add_remote(path: String, name: String, url: String) -> CommandResponse<CommandResult> {
    add_remote_core(&path, name, url).map_err(|err| err.command())
}

#[tauri::command]
fn update_remote(
    path: String,
    name: String,
    url: String,
    push_url: Option<String>,
) -> CommandResponse<CommandResult> {
    update_remote_core(&path, name, url, push_url).map_err(|err| err.command())
}

#[tauri::command]
fn delete_remote(path: String, name: String) -> CommandResponse<CommandResult> {
    delete_remote_core(&path, name).map_err(|err| err.command())
}

#[tauri::command]
fn list_commits(
    path: String,
    limit: Option<usize>,
    branch: Option<String>,
    query: Option<String>,
    author: Option<String>,
    authors: Option<Vec<String>>,
    path_filter: Option<String>,
    path_filters: Option<Vec<String>>,
) -> CommandResponse<Vec<CommitSummary>> {
    list_commits_filtered_multi_core(
        &path,
        limit,
        branch,
        query,
        merge_optional_filters(author, authors),
        merge_optional_filters(path_filter, path_filters),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn commit_details(path: String, oid: String) -> CommandResponse<CommitDetails> {
    commit_details_core(&path, oid).map_err(|err| err.command())
}

#[tauri::command]
fn commit_file_diff(
    path: String,
    oid: String,
    file_path: String,
    mode: Option<String>,
) -> CommandResponse<DiffResponse> {
    commit_file_diff_core(&path, oid, file_path, mode).map_err(|err| err.command())
}

#[tauri::command]
fn file_history(
    path: String,
    file_path: String,
    limit: Option<usize>,
) -> CommandResponse<Vec<FileHistoryEntry>> {
    file_history_core(&path, file_path, limit).map_err(|err| err.command())
}

#[tauri::command]
fn blame_file(path: String, file_path: String) -> CommandResponse<Vec<BlameLine>> {
    blame_file_core(&path, file_path).map_err(|err| err.command())
}

#[tauri::command]
fn compare_refs(path: String, left: String, right: String) -> CommandResponse<RefComparison> {
    compare_refs_core(&path, left, right).map_err(|err| err.command())
}

#[tauri::command]
fn list_branches(path: String) -> CommandResponse<BranchList> {
    list_branches_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn checkout_branch(path: String, name: String) -> CommandResponse<CommandResult> {
    checkout_branch_core(&path, name).map_err(|err| err.command())
}

#[tauri::command]
fn checkout_revision(path: String, revision: String) -> CommandResponse<CommandResult> {
    checkout_revision_core(&path, revision).map_err(|err| err.command())
}

#[tauri::command]
fn checkout_remote_branch(
    path: String,
    remote_branch: String,
    local_name: Option<String>,
) -> CommandResponse<CommandResult> {
    checkout_remote_branch_core(&path, remote_branch, local_name).map_err(|err| err.command())
}

#[tauri::command]
fn create_branch(
    path: String,
    name: String,
    checkout: Option<bool>,
    start_point: Option<String>,
) -> CommandResponse<CommandResult> {
    create_branch_core(&path, name, checkout.unwrap_or(true), start_point)
        .map_err(|err| err.command())
}

#[tauri::command]
fn rename_branch(
    path: String,
    old_name: String,
    new_name: String,
) -> CommandResponse<CommandResult> {
    rename_branch_core(&path, old_name, new_name).map_err(|err| err.command())
}

#[tauri::command]
fn cleanup_merged_branches(path: String, target: Option<String>) -> CommandResponse<CommandResult> {
    cleanup_merged_branches_core(&path, target).map_err(|err| err.command())
}

#[tauri::command]
fn delete_branch(
    path: String,
    name: String,
    force: Option<bool>,
) -> CommandResponse<CommandResult> {
    delete_branch_core(&path, name, force.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn delete_remote_branch(path: String, remote_branch: String) -> CommandResponse<CommandResult> {
    delete_remote_branch_core(&path, remote_branch).map_err(|err| err.command())
}

#[tauri::command]
fn set_branch_upstream(
    path: String,
    branch_name: String,
    upstream: Option<String>,
) -> CommandResponse<CommandResult> {
    set_branch_upstream_core(&path, branch_name, upstream).map_err(|err| err.command())
}

#[tauri::command]
fn create_tag(
    path: String,
    name: String,
    target: Option<String>,
    annotated: Option<bool>,
    message: Option<String>,
) -> CommandResponse<CommandResult> {
    create_tag_core(&path, name, target, annotated.unwrap_or(false), message)
        .map_err(|err| err.command())
}

#[tauri::command]
fn delete_tag(path: String, name: String) -> CommandResponse<CommandResult> {
    delete_tag_core(&path, name).map_err(|err| err.command())
}

#[tauri::command]
fn push_tag(
    path: String,
    remote_name: Option<String>,
    name: String,
) -> CommandResponse<CommandResult> {
    push_tag_core(&path, remote_name, name).map_err(|err| err.command())
}

#[tauri::command]
fn delete_remote_tag(
    path: String,
    remote_name: Option<String>,
    name: String,
) -> CommandResponse<CommandResult> {
    delete_remote_tag_core(&path, remote_name, name).map_err(|err| err.command())
}

#[tauri::command]
fn merge_branch(
    path: String,
    target: String,
    no_ff: Option<bool>,
    no_commit: Option<bool>,
    squash: Option<bool>,
) -> CommandResponse<CommandResult> {
    merge_branch_core(
        &path,
        target,
        no_ff.unwrap_or(false),
        no_commit.unwrap_or(false),
        squash.unwrap_or(false),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn rebase_branch(
    path: String,
    target: String,
    autostash: Option<bool>,
) -> CommandResponse<CommandResult> {
    rebase_branch_core(&path, target, autostash.unwrap_or(true)).map_err(|err| err.command())
}

#[tauri::command]
fn rebase_advanced(
    path: String,
    target: Option<String>,
    source_branch: Option<String>,
    onto: Option<String>,
    autostash: Option<bool>,
    interactive: Option<bool>,
    autosquash: Option<bool>,
    rebase_merges: Option<bool>,
    keep_empty: Option<bool>,
    root: Option<bool>,
    update_refs: Option<bool>,
) -> CommandResponse<CommandResult> {
    rebase_advanced_core(
        &path,
        target,
        source_branch,
        onto,
        autostash.unwrap_or(true),
        interactive.unwrap_or(false),
        autosquash.unwrap_or(false),
        rebase_merges.unwrap_or(false),
        keep_empty.unwrap_or(false),
        root.unwrap_or(false),
        update_refs.unwrap_or(false),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn cherry_pick_commit(path: String, oid: String) -> CommandResponse<CommandResult> {
    cherry_pick_commit_core(&path, oid).map_err(|err| err.command())
}

#[tauri::command]
fn cherry_pick_files(
    path: String,
    oid: String,
    files: Vec<String>,
) -> CommandResponse<CommandResult> {
    cherry_pick_files_core(&path, oid, files).map_err(|err| err.command())
}

#[tauri::command]
fn revert_commit_files(
    path: String,
    oid: String,
    files: Vec<String>,
) -> CommandResponse<CommandResult> {
    revert_commit_files_core(&path, oid, files).map_err(|err| err.command())
}

#[tauri::command]
fn revert_commit(
    path: String,
    oid: String,
    no_commit: Option<bool>,
) -> CommandResponse<CommandResult> {
    revert_commit_core(&path, oid, no_commit.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn reset_to_commit(path: String, oid: String, mode: String) -> CommandResponse<CommandResult> {
    reset_to_commit_core(&path, oid, mode).map_err(|err| err.command())
}

#[tauri::command]
fn undo_last_commit(path: String, keep_staged: Option<bool>) -> CommandResponse<CommandResult> {
    undo_last_commit_core(&path, keep_staged.unwrap_or(true)).map_err(|err| err.command())
}

#[tauri::command]
fn fixup_commit(path: String, oid: String, squash: Option<bool>) -> CommandResponse<CommandResult> {
    fixup_commit_core(&path, oid, squash.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn drop_commit(path: String, oid: String) -> CommandResponse<CommandResult> {
    drop_commit_core(&path, oid).map_err(|err| err.command())
}

#[tauri::command]
fn push_commit(
    path: String,
    remote_name: Option<String>,
    oid: String,
    target_branch: Option<String>,
) -> CommandResponse<CommandResult> {
    push_commit_core(&path, remote_name, oid, target_branch).map_err(|err| err.command())
}

#[tauri::command]
fn create_patch(
    path: String,
    paths: Vec<String>,
    staged: Option<bool>,
) -> CommandResponse<CommandResult> {
    create_patch_core(&path, paths, staged.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn apply_patch(
    path: String,
    patch: String,
    index: Option<bool>,
    three_way: Option<bool>,
) -> CommandResponse<CommandResult> {
    apply_patch_core(
        &path,
        patch,
        index.unwrap_or(false),
        three_way.unwrap_or(true),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn list_worktrees(path: String) -> CommandResponse<Vec<WorktreeInfo>> {
    list_worktrees_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn create_worktree(
    path: String,
    worktree_path: String,
    branch: Option<String>,
    start_point: Option<String>,
    detach: Option<bool>,
) -> CommandResponse<CommandResult> {
    create_worktree_core(
        &path,
        worktree_path,
        branch,
        start_point,
        detach.unwrap_or(false),
    )
    .map_err(|err| err.command())
}

#[tauri::command]
fn remove_worktree(
    path: String,
    worktree_path: String,
    force: Option<bool>,
) -> CommandResponse<CommandResult> {
    remove_worktree_core(&path, worktree_path, force.unwrap_or(false)).map_err(|err| err.command())
}

#[tauri::command]
fn list_stashes(path: String) -> CommandResponse<Vec<StashInfo>> {
    list_stashes_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn stash_action(path: String, stash_ref: String, action: String) -> CommandResponse<CommandResult> {
    stash_action_core(&path, stash_ref, action).map_err(|err| err.command())
}

#[tauri::command]
fn clear_stashes(path: String) -> CommandResponse<CommandResult> {
    clear_stashes_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn list_submodules(path: String) -> CommandResponse<Vec<SubmoduleInfo>> {
    list_submodules_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn update_submodules(
    path: String,
    init: Option<bool>,
    recursive: Option<bool>,
) -> CommandResponse<CommandResult> {
    update_submodules_core(&path, init.unwrap_or(true), recursive.unwrap_or(true))
        .map_err(|err| err.command())
}

#[tauri::command]
fn lfs_status(path: String) -> CommandResponse<CommandResult> {
    lfs_status_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn commit_message_history(path: String, limit: Option<usize>) -> CommandResponse<Vec<String>> {
    commit_message_history_core(&path, limit).map_err(|err| err.command())
}

#[tauri::command]
fn operation_state(path: String) -> CommandResponse<GitOperationState> {
    operation_state_core(&path).map_err(|err| err.command())
}

#[tauri::command]
fn operation_control(path: String, action: String) -> CommandResponse<CommandResult> {
    operation_control_core(&path, action).map_err(|err| err.command())
}

#[tauri::command]
fn conflict_details(path: String, file_path: String) -> CommandResponse<ConflictDetails> {
    conflict_details_core(&path, file_path).map_err(|err| err.command())
}

#[tauri::command]
fn resolve_conflict_file(
    path: String,
    file_path: String,
    side: String,
) -> CommandResponse<CommandResult> {
    resolve_conflict_file_core(&path, file_path, side).map_err(|err| err.command())
}

#[tauri::command]
fn resolve_conflict_block(
    path: String,
    file_path: String,
    block_index: usize,
    side: String,
) -> CommandResponse<CommandResult> {
    resolve_conflict_block_core(&path, file_path, block_index, side).map_err(|err| err.command())
}

#[tauri::command]
fn mark_conflict_resolved(path: String, file_path: String) -> CommandResponse<CommandResult> {
    mark_conflict_resolved_core(&path, file_path).map_err(|err| err.command())
}

#[tauri::command]
fn save_conflict_result(
    path: String,
    file_path: String,
    content: String,
    mark_resolved: Option<bool>,
) -> CommandResponse<CommandResult> {
    save_conflict_result_core(&path, file_path, content, mark_resolved.unwrap_or(false))
        .map_err(|err| err.command())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_repo,
            filter_project_directories,
            open_project_directory,
            open_project_terminal,
            init_repository,
            clone_repository,
            unshallow_repository,
            repo_status,
            list_project_files,
            read_project_file,
            save_project_file,
            create_project_file,
            create_project_directory,
            rename_project_entry,
            delete_project_entry,
            copy_project_entry,
            move_project_entry,
            get_diff,
            stage_paths,
            unstage_paths,
            stage_hunks,
            discard_changes,
            shelve_changes,
            unshelve,
            delete_shelf,
            list_shelves,
            commit,
            fetch,
            pull_preflight,
            pull,
            push,
            add_remote,
            update_remote,
            delete_remote,
            branch_summary,
            list_commits,
            commit_details,
            commit_file_diff,
            file_history,
            blame_file,
            compare_refs,
            list_branches,
            checkout_branch,
            checkout_revision,
            checkout_remote_branch,
            create_branch,
            rename_branch,
            cleanup_merged_branches,
            delete_branch,
            delete_remote_branch,
            set_branch_upstream,
            create_tag,
            delete_tag,
            push_tag,
            delete_remote_tag,
            merge_branch,
            rebase_branch,
            rebase_advanced,
            cherry_pick_commit,
            cherry_pick_files,
            revert_commit_files,
            revert_commit,
            reset_to_commit,
            undo_last_commit,
            fixup_commit,
            drop_commit,
            push_commit,
            create_patch,
            apply_patch,
            list_worktrees,
            create_worktree,
            remove_worktree,
            list_stashes,
            stash_action,
            clear_stashes,
            list_submodules,
            update_submodules,
            lfs_status,
            commit_message_history,
            operation_state,
            operation_control,
            conflict_details,
            resolve_conflict_file,
            resolve_conflict_block,
            mark_conflict_resolved,
            save_conflict_result
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
