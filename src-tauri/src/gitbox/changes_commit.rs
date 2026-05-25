pub fn repo_status_core(path: &str, include_ignored: bool) -> Result<RepoStatus, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let mut args = vec![
        "status".to_string(),
        "--porcelain=v1".to_string(),
        "-z".to_string(),
        "--untracked-files=all".to_string(),
        "--no-renames".to_string(),
    ];
    if include_ignored {
        args.push("--ignored".to_string());
    }

    let output = run_git(&workdir, args, None)?;
    let (mut files, counts) = parse_porcelain_status(&output);

    files.sort_by(|left, right| left.path.cmp(&right.path));
    let clean = files.is_empty();
    Ok(RepoStatus {
        repo: repository_info(&repo)?,
        branch: branch_summary_for_repo(&repo, include_ignored, Some(clean))?,
        files,
        counts,
    })
}

pub fn branch_summary_core(
    path: &str,
    include_ignored: bool,
) -> Result<BranchSummary, GitboxError> {
    let repo = Repository::discover(path)?;
    branch_summary_for_repo(&repo, include_ignored, None)
}

fn branch_summary_for_repo(
    repo: &Repository,
    include_ignored: bool,
    clean_override: Option<bool>,
) -> Result<BranchSummary, GitboxError> {
    let head = repo.head().ok();
    let current_branch = current_branch_name(repo);
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
        clean: match clean_override {
            Some(clean) => clean,
            None => repo_is_clean(repo, include_ignored)?,
        },
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
    opts.include_untracked(true).include_typechange(true);

    let normalized_path = match file_path {
        Some(path) if !path.trim().is_empty() => {
            let rel = normalize_repo_path(&repo, &path)?;
            let pathspec = repo_path_string(&rel);
            opts.pathspec(&pathspec);
            Some(pathspec)
        }
        _ => None,
    };
    if !staged && normalized_path.is_some() {
        opts.show_untracked_content(true);
    }

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

    let (old_text, new_text) = match normalized_path.as_deref() {
        Some(pathspec) if staged => (
            read_treeish_file_text(&repo, "HEAD", pathspec)?,
            read_index_file_text(&repo, pathspec)?,
        ),
        Some(pathspec) => (
            read_index_file_text(&repo, pathspec)?,
            read_workdir_file_text(&repo, pathspec)?,
        ),
        None => (None, None),
    };

    Ok(DiffResponse {
        path: normalized_path,
        staged,
        old_text,
        new_text,
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
    commit_with_full_options_and_worktree_core(
        path, message, amend, sign_off, gpg_sign, author, false,
    )
}

pub fn commit_with_full_options_and_worktree_core(
    path: &str,
    message: String,
    amend: bool,
    sign_off: bool,
    gpg_sign: bool,
    author: Option<String>,
    include_worktree: bool,
) -> Result<CommitResult, GitboxError> {
    commit_with_full_options_and_selection_core(
        path,
        message,
        amend,
        sign_off,
        gpg_sign,
        author,
        include_worktree,
        None,
    )
}

pub fn commit_with_full_options_and_selection_core(
    path: &str,
    message: String,
    amend: bool,
    sign_off: bool,
    gpg_sign: bool,
    author: Option<String>,
    include_worktree: bool,
    selected_paths: Option<Vec<String>>,
) -> Result<CommitResult, GitboxError> {
    if message.trim().is_empty() {
        return Err(GitboxError::Message("提交信息不能为空".to_string()));
    }

    let repo = Repository::discover(path)?;
    if repo.path().join("MERGE_HEAD").exists() {
        return commit_merge_with_git(
            path,
            &repo,
            message,
            amend,
            sign_off,
            gpg_sign,
            author,
            include_worktree,
            selected_paths,
        );
    }

    if let Some(selected_paths) = selected_paths {
        return commit_selected_paths_with_git(
            path,
            &repo,
            message,
            amend,
            sign_off,
            gpg_sign,
            author,
            selected_paths,
        );
    }

    if include_worktree {
        ensure_unresolved_worktree_conflicts_have_no_markers(&repo, "提交")?;
        stage_worktree_changes_for_commit(&repo)?;
    }

    ensure_index_ready_for_commit(&repo, None, "提交")?;

    let staged_diff = get_diff_core(path, None, true)?;
    if !amend && staged_diff.text.trim().is_empty() {
        let message = if include_worktree {
            "没有可提交的变更"
        } else {
            "没有已暂存的变更可提交"
        };
        return Err(GitboxError::Message(message.to_string()));
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

fn stage_worktree_changes_for_commit(repo: &Repository) -> Result<(), GitboxError> {
    let workdir = repo_workdir(repo)?;
    run_git(&workdir, vec!["add".to_string(), "-A".to_string()], None)?;
    Ok(())
}

fn ensure_index_ready_for_commit(
    repo: &Repository,
    marker_paths: Option<&[String]>,
    action: &str,
) -> Result<(), GitboxError> {
    ensure_no_unresolved_index_conflicts(repo, action)?;
    ensure_staged_diff_has_no_conflict_markers(repo, marker_paths, action)?;
    Ok(())
}

fn unresolved_conflict_paths(repo: &Repository) -> Result<Vec<String>, GitboxError> {
    let mut opts = StatusOptions::new();
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut paths = Vec::new();

    for entry in statuses.iter() {
        if !entry.status().contains(Status::CONFLICTED) {
            continue;
        }
        let path = entry.path().unwrap_or("<unknown>").to_string();
        if !paths.iter().any(|item| item == &path) {
            paths.push(path);
        }
    }

    paths.sort();
    Ok(paths)
}

fn ensure_no_unresolved_index_conflicts(
    repo: &Repository,
    action: &str,
) -> Result<(), GitboxError> {
    let paths = unresolved_conflict_paths(repo)?;

    if paths.is_empty() {
        return Ok(());
    }

    let mut message =
        format!("仍有未解决的冲突文件，不能{action}。请先在合并工具中处理冲突并标记为已解决。");
    append_limited_entries(&mut message, "受影响文件", &paths);
    Err(GitboxError::Message(message))
}

fn ensure_unresolved_worktree_conflicts_have_no_markers(
    repo: &Repository,
    action: &str,
) -> Result<(), GitboxError> {
    let paths = unresolved_conflict_paths(repo)?;
    ensure_worktree_paths_have_no_conflict_markers(repo, &paths, action)
}

fn ensure_staged_diff_has_no_conflict_markers(
    repo: &Repository,
    paths: Option<&[String]>,
    action: &str,
) -> Result<(), GitboxError> {
    let workdir = repo_workdir(repo)?;
    let mut args = vec![
        "diff".to_string(),
        "--check".to_string(),
        "--cached".to_string(),
    ];
    if let Some(paths) = paths.filter(|paths| !paths.is_empty()) {
        args.push("--".to_string());
        args.extend(paths.iter().cloned());
    }

    let output = run_git_raw(&workdir, args, None)?;
    let marker_lines = conflict_marker_lines_from_diff_check(&output.combined_output());
    if marker_lines.is_empty() {
        return Ok(());
    }

    Err(conflict_marker_error(action, &marker_lines))
}

fn ensure_worktree_paths_have_no_conflict_markers(
    repo: &Repository,
    paths: &[String],
    action: &str,
) -> Result<(), GitboxError> {
    let mut marker_lines = Vec::new();
    for path in paths {
        let Some(content) = read_workdir_file_text(repo, path)? else {
            continue;
        };
        marker_lines.extend(conflict_marker_lines_from_content(path, &content));
    }

    if marker_lines.is_empty() {
        return Ok(());
    }

    Err(conflict_marker_error(action, &marker_lines))
}

fn conflict_marker_lines_from_content(path: &str, content: &str) -> Vec<String> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let has_marker = line.starts_with("<<<<<<< ")
                || line.starts_with("||||||| ")
                || line == "======="
                || line.starts_with(">>>>>>> ");
            has_marker.then(|| format!("{path}:{}: leftover conflict marker", index + 1))
        })
        .collect()
}

fn conflict_marker_lines_from_diff_check(output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.contains("leftover conflict marker") {
            continue;
        }
        if !lines.iter().any(|item| item == line) {
            lines.push(line.to_string());
        }
    }
    lines
}

fn conflict_marker_error(action: &str, entries: &[String]) -> GitboxError {
    let mut message = format!(
        "仍有文件包含冲突标记，不能{action}。请先删除 <<<<<<<、=======、>>>>>>> 标记并重新暂存。"
    );
    append_limited_entries(&mut message, "受影响位置", entries);
    GitboxError::Message(message)
}

fn append_limited_entries(message: &mut String, title: &str, entries: &[String]) {
    if entries.is_empty() {
        return;
    }

    message.push_str("\n\n");
    message.push_str(title);
    message.push('：');
    for entry in entries.iter().take(12) {
        message.push_str("\n- ");
        message.push_str(entry);
    }
    if entries.len() > 12 {
        message.push_str(&format!("\n- 另有 {} 项", entries.len() - 12));
    }
}

const EMPTY_TREE_OID: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

fn ensure_no_pending_operation_for_action(path: &str, action: &str) -> Result<(), GitboxError> {
    let state = operation_state_core(path)?;
    if state.operation.is_none() && state.conflicted_paths.is_empty() {
        return Ok(());
    }

    let operation = state
        .operation
        .as_deref()
        .map(operation_label)
        .unwrap_or("Git 操作");
    let mut message =
        format!("当前{operation}尚未完成，不能{action}。请先解决冲突并完成或终止该操作。");
    append_limited_entries(&mut message, "受影响文件", &state.conflicted_paths);
    Err(GitboxError::Message(message))
}

fn ensure_outgoing_diff_has_no_conflict_markers(
    repo: &Repository,
    workdir: &Path,
    remote_name: &str,
    target_branch: &str,
) -> Result<(), GitboxError> {
    let remote_ref = format!("refs/remotes/{remote_name}/{target_branch}");
    let range = if repo.find_reference(&remote_ref).is_ok() {
        format!("{remote_ref}..HEAD")
    } else {
        format!("{EMPTY_TREE_OID}..HEAD")
    };
    let output = run_git_raw(
        workdir,
        vec!["diff".to_string(), "--check".to_string(), range],
        None,
    )?;
    let marker_lines = conflict_marker_lines_from_diff_check(&output.combined_output());
    if marker_lines.is_empty() {
        return Ok(());
    }

    Err(conflict_marker_error("推送", &marker_lines))
}

fn commit_merge_with_git(
    path: &str,
    repo: &Repository,
    message: String,
    amend: bool,
    sign_off: bool,
    gpg_sign: bool,
    author: Option<String>,
    include_worktree: bool,
    selected_paths: Option<Vec<String>>,
) -> Result<CommitResult, GitboxError> {
    if amend {
        return Err(GitboxError::Message(
            "合并进行中不能使用“修正上次提交”，请完成或终止当前合并".to_string(),
        ));
    }

    let workdir = repo_workdir(repo)?;
    if let Some(selected_paths) = selected_paths {
        let selected_paths = clean_selected_paths(repo, selected_paths)?;
        if !selected_paths.is_empty() {
            ensure_worktree_paths_have_no_conflict_markers(repo, &selected_paths, "提交")?;
            run_git(
                &workdir,
                git_args_with_paths(&["add", "-A"], &selected_paths),
                None,
            )?;
        }
    } else if include_worktree {
        ensure_unresolved_worktree_conflicts_have_no_markers(repo, "提交")?;
        stage_worktree_changes_for_commit(repo)?;
    }

    ensure_index_ready_for_commit(repo, None, "提交")?;

    let mut args = vec!["commit".to_string()];
    if sign_off {
        args.push("--signoff".to_string());
    }
    if gpg_sign {
        args.push("-S".to_string());
    }
    if let Some(author) = clean_optional_arg(author) {
        args.push("--author".to_string());
        args.push(author);
    }
    args.push("-m".to_string());
    args.push(message.trim().to_string());

    run_git(&workdir, args, None)?;
    Ok(CommitResult {
        oid: repo_head_oid(path)?,
        summary: branch_summary_core(path, false)?,
    })
}

fn commit_selected_paths_with_git(
    path: &str,
    repo: &Repository,
    message: String,
    amend: bool,
    sign_off: bool,
    gpg_sign: bool,
    author: Option<String>,
    selected_paths: Vec<String>,
) -> Result<CommitResult, GitboxError> {
    let selected_paths = clean_selected_paths(repo, selected_paths)?;
    if selected_paths.is_empty() {
        return Err(GitboxError::Message("请选择要提交的文件".to_string()));
    }

    let workdir = repo_workdir(repo)?;
    ensure_worktree_paths_have_no_conflict_markers(repo, &selected_paths, "提交")?;
    run_git(
        &workdir,
        git_args_with_paths(&["add", "-A"], &selected_paths),
        None,
    )?;

    ensure_no_unresolved_index_conflicts(repo, "提交")?;
    ensure_staged_diff_has_no_conflict_markers(repo, Some(&selected_paths), "提交")?;

    let mut args = vec!["commit".to_string(), "--only".to_string()];
    if amend {
        args.push("--amend".to_string());
    }
    if sign_off {
        args.push("--signoff".to_string());
    }
    if gpg_sign {
        args.push("-S".to_string());
    }
    if let Some(author) = clean_optional_arg(author) {
        args.push("--author".to_string());
        args.push(author);
    }
    args.push("-m".to_string());
    args.push(message.trim().to_string());
    args.push("--".to_string());
    args.extend(selected_paths);

    run_git(&workdir, args, None)?;
    Ok(CommitResult {
        oid: repo_head_oid(path)?,
        summary: branch_summary_core(path, false)?,
    })
}
