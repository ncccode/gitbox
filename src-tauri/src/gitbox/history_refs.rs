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
            "-z".to_string(),
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

    let parent_treeish = if matches!(diff_mode.as_str(), "commit" | "parent-worktree") {
        Some(parent_or_empty_tree(&repo, &oid)?)
    } else {
        None
    };

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
            parent_treeish.clone().unwrap_or_default(),
            "--".to_string(),
            pathspec.clone(),
        ],
        _ => return Err(GitboxError::Message("不支持的文件差异比较方式".to_string())),
    };

    let text = run_git(&workdir, args, None)?;
    let (old_text, new_text) = match diff_mode.as_str() {
        "commit" => (
            parent_treeish
                .as_deref()
                .map(|treeish| read_treeish_file_text(&repo, treeish, &pathspec))
                .transpose()?
                .flatten(),
            read_treeish_file_text(&repo, &oid, &pathspec)?,
        ),
        "worktree" => (
            read_treeish_file_text(&repo, &oid, &pathspec)?,
            read_workdir_file_text(&repo, &pathspec)?,
        ),
        "parent-worktree" => (
            parent_treeish
                .as_deref()
                .map(|treeish| read_treeish_file_text(&repo, treeish, &pathspec))
                .transpose()?
                .flatten(),
            read_workdir_file_text(&repo, &pathspec)?,
        ),
        _ => (None, None),
    };

    Ok(DiffResponse {
        path: Some(pathspec),
        staged: false,
        old_text,
        new_text,
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
            "-z".to_string(),
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
    if let Some(current_name) = current.as_deref().filter(|name| *name != "HEAD") {
        if !branches
            .iter()
            .any(|branch| branch.branch_type == "local" && branch.name == current_name)
        {
            let target = repo
                .head()
                .ok()
                .and_then(|head| head.target())
                .map(|oid| oid.to_string());
            branches.push(BranchInfo {
                name: current_name.to_string(),
                full_name: format!("refs/heads/{current_name}"),
                branch_type: "local".to_string(),
                current: true,
                upstream: None,
                target,
                ahead: 0,
                behind: 0,
            });
        }
    }
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
