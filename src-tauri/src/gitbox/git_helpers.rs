fn repository_info(repo: &Repository) -> Result<RepositoryInfo, GitboxError> {
    let workdir = repo
        .workdir()
        .map(|path| path_string(&normalize_existing_path(path)));
    let root = normalize_existing_path(&repo_root(repo)?);
    let branch = current_branch_name(repo);
    let head = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string());

    Ok(RepositoryInfo {
        path: path_string(&root),
        workdir,
        git_dir: path_string(&normalize_existing_path(repo.path())),
        is_bare: repo.is_bare(),
        branch,
        head,
        remotes: remote_infos(repo)?,
    })
}

fn current_branch_name(repo: &Repository) -> Option<String> {
    if let Ok(head) = repo.head() {
        if let Some(name) = head
            .symbolic_target()
            .and_then(|target| target.strip_prefix("refs/heads/"))
        {
            return Some(name.to_string());
        }
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
    if output.contains('\0') {
        return parse_name_status_nul(output);
    }

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

fn parse_name_status_nul(output: &str) -> Vec<CommitFileChange> {
    let mut files = Vec::new();
    let mut parts = output.split('\0').filter(|part| !part.is_empty());

    while let Some(status) = parts.next() {
        let Some(first_path) = parts.next() else {
            break;
        };
        let (old_path, path) = if status.starts_with('R') || status.starts_with('C') {
            let Some(next_path) = parts.next() else {
                break;
            };
            (Some(first_path.to_string()), next_path.to_string())
        } else {
            (None, first_path.to_string())
        };

        if path.is_empty() {
            continue;
        }
        files.push(CommitFileChange {
            path,
            old_path,
            status: status.to_string(),
        });
    }

    files
}

fn parse_porcelain_status(output: &str) -> (Vec<ChangedFile>, StatusCounts) {
    let mut files = Vec::new();
    let mut counts = StatusCounts::default();
    let mut entries = output.split('\0').filter(|part| !part.is_empty());

    while let Some(entry) = entries.next() {
        if entry.len() < 4 {
            continue;
        }

        let bytes = entry.as_bytes();
        let index = bytes[0] as char;
        let worktree = bytes[1] as char;
        let path = entry[3..].to_string();
        if path.is_empty() {
            continue;
        }

        let old_path = if matches!(index, 'R' | 'C') || matches!(worktree, 'R' | 'C') {
            entries
                .next()
                .filter(|old| !old.is_empty() && *old != path)
                .map(ToOwned::to_owned)
        } else {
            None
        };
        let conflicted = porcelain_status_conflicted(index, worktree);
        let untracked = index == '?' && worktree == '?';
        let ignored = index == '!' && worktree == '!';
        let staged = !conflicted && porcelain_index_changed(index);
        let unstaged = untracked || (!conflicted && porcelain_worktree_changed(worktree));

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

        files.push(ChangedFile {
            path,
            old_path,
            kind: porcelain_status_kind(index, worktree, conflicted, untracked, ignored),
            staged,
            unstaged,
            untracked,
            ignored,
            conflicted,
        });
    }

    (files, counts)
}

fn porcelain_status_conflicted(index: char, worktree: char) -> bool {
    matches!(
        (index, worktree),
        ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
    )
}

fn porcelain_index_changed(status: char) -> bool {
    matches!(status, 'A' | 'M' | 'D' | 'R' | 'C' | 'T')
}

fn porcelain_worktree_changed(status: char) -> bool {
    matches!(status, 'M' | 'D' | 'R' | 'C' | 'T')
}

fn porcelain_status_kind(
    index: char,
    worktree: char,
    conflicted: bool,
    untracked: bool,
    ignored: bool,
) -> String {
    if conflicted {
        return "conflicted".to_string();
    }

    let mut parts = Vec::new();
    if untracked || matches!(index, 'A' | 'C') || matches!(worktree, 'A' | 'C') {
        parts.push("added");
    }
    if matches!(index, 'M') || matches!(worktree, 'M') {
        parts.push("modified");
    }
    if matches!(index, 'D') || matches!(worktree, 'D') {
        parts.push("deleted");
    }
    if matches!(index, 'R') || matches!(worktree, 'R') {
        parts.push("renamed");
    }
    if matches!(index, 'T') || matches!(worktree, 'T') {
        parts.push("typechange");
    }
    if ignored {
        parts.push("ignored");
    }
    if parts.is_empty() {
        parts.push("unknown");
    }
    parts.join("|")
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

#[cfg(target_os = "macos")]
fn open_system_file_manager(path: &Path) -> Result<(), GitboxError> {
    let status = Command::new("open").arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(GitboxError::Message("无法打开系统文件管理器".to_string()))
    }
}

#[cfg(windows)]
fn open_system_file_manager(path: &Path) -> Result<(), GitboxError> {
    let mut command = Command::new("explorer");
    command
        .creation_flags(CREATE_NO_WINDOW)
        .arg(path_string(path));
    command.spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_system_file_manager(path: &Path) -> Result<(), GitboxError> {
    let openers: [&str; 3] = ["xdg-open", "gio", "kde-open"];
    for program in openers {
        let mut command = Command::new(program);
        if program == "gio" {
            command.arg("open");
        }
        command.arg(path);

        match command.spawn() {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        }
    }

    Err(GitboxError::Message(
        "无法找到可用的系统文件管理器".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn open_system_terminal(path: &Path) -> Result<(), GitboxError> {
    let status = Command::new("open")
        .args(["-a", "Terminal"])
        .arg(path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(GitboxError::Message("无法打开系统终端".to_string()))
    }
}

#[cfg(windows)]
fn open_system_terminal(path: &Path) -> Result<(), GitboxError> {
    let mut command = Command::new("cmd");
    command
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/C", "start", "", "cmd", "/K", "cd", "/d"])
        .arg(path_string(path));
    command.spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_system_terminal(path: &Path) -> Result<(), GitboxError> {
    let terminals: [(&str, &[&str]); 8] = [
        ("x-terminal-emulator", &[]),
        ("gnome-terminal", &["--working-directory"]),
        ("konsole", &["--workdir"]),
        ("xfce4-terminal", &["--working-directory"]),
        ("mate-terminal", &["--working-directory"]),
        ("kitty", &["--directory"]),
        ("alacritty", &["--working-directory"]),
        ("xterm", &[]),
    ];

    for (program, args) in terminals {
        let mut command = Command::new(program);
        command.current_dir(path);
        if !args.is_empty() {
            command.args(args).arg(path);
        }

        match command.spawn() {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        }
    }

    Err(GitboxError::Message("无法找到可用的系统终端".to_string()))
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

fn current_pull_remote_and_target(
    repo: &Repository,
    remote_name: Option<String>,
) -> Result<(String, String), GitboxError> {
    let branch_name = current_branch_name(repo)
        .ok_or_else(|| GitboxError::Message("当前 HEAD 不是本地分支，不能直接拉取".to_string()))?;
    let configured_upstream = repo
        .find_branch(&branch_name, BranchType::Local)
        .ok()
        .and_then(|branch| branch.upstream().ok())
        .and_then(|branch| branch.name().ok().flatten().map(ToOwned::to_owned));

    if let Some(remote) = clean_optional_arg(remote_name) {
        let target = configured_upstream
            .filter(|upstream| upstream_remote_name(upstream).as_deref() == Some(remote.as_str()))
            .unwrap_or_else(|| format!("{remote}/{branch_name}"));
        return Ok((remote, target));
    }

    if let Some(upstream) = configured_upstream {
        let remote = upstream_remote_name(&upstream).unwrap_or_else(|| "origin".to_string());
        return Ok((remote, upstream));
    }

    Ok(("origin".to_string(), format!("origin/{branch_name}")))
}

fn merge_base_or_none(
    repo: &Repository,
    left: Oid,
    right: Oid,
) -> Result<Option<Oid>, GitboxError> {
    match repo.merge_base(left, right) {
        Ok(oid) => Ok(Some(oid)),
        Err(error) if error.code() == ErrorCode::NotFound => Ok(None),
        Err(error) => Err(GitboxError::Git(error)),
    }
}

fn unrelated_histories_pull_message(target: &str, remote: &str) -> String {
    format!(
        "当前分支与 {target} 没有共同提交历史，无法直接拉取。请确认远程 {remote} 或上游分支是否选错；若确实要合并两个无关仓库，请在命令行使用 --allow-unrelated-histories。"
    )
}

fn build_pull_preflight(
    repo: &Repository,
    remote: String,
    target: String,
) -> Result<PullPreflight, GitboxError> {
    let current_branch = current_branch_name(repo);
    let head_oid = repo
        .head()?
        .target()
        .ok_or_else(|| GitboxError::Message("当前 HEAD 无法拉取".to_string()))?;
    let target_oid = repo
        .revparse_single(&target)
        .and_then(|object| object.peel_to_commit())
        .map(|commit| commit.id())
        .map_err(|_| GitboxError::Message(format!("找不到远程分支 {target}")))?;

    let merge_base = if head_oid == target_oid {
        head_oid
    } else {
        merge_base_or_none(repo, head_oid, target_oid)?.ok_or_else(|| {
            GitboxError::Message(unrelated_histories_pull_message(&target, &remote))
        })?
    };
    let up_to_date = head_oid == target_oid;
    let fast_forward = !up_to_date && merge_base == head_oid;
    let diverged = !up_to_date && merge_base != head_oid && merge_base != target_oid;
    let remote_changed_paths = if up_to_date || merge_base == target_oid {
        Vec::new()
    } else {
        changed_paths_between(repo, merge_base, target_oid)?
    };
    let local_changed_paths = local_uncommitted_paths(repo)?;
    let local_set = local_changed_paths
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let overlapping_paths = sorted_unique(
        remote_changed_paths
            .iter()
            .filter(|path| local_set.contains(path.as_str()))
            .cloned(),
    );
    let needs_confirmation = !overlapping_paths.is_empty();

    Ok(PullPreflight {
        remote,
        target,
        current_branch,
        head: Some(head_oid.to_string()),
        target_oid: Some(target_oid.to_string()),
        up_to_date,
        fast_forward,
        diverged,
        local_changed_paths,
        remote_changed_paths,
        overlapping_paths,
        needs_confirmation,
    })
}

fn local_uncommitted_paths(repo: &Repository) -> Result<Vec<String>, GitboxError> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut paths = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        if status == Status::CURRENT || status.contains(Status::IGNORED) {
            continue;
        }

        if let Some(delta) = entry.head_to_index() {
            collect_delta_paths(delta, &mut paths);
        }
        if let Some(delta) = entry.index_to_workdir() {
            collect_delta_paths(delta, &mut paths);
        }
        if let Some(path) = entry.path() {
            paths.push(path.to_string());
        }
    }

    Ok(sorted_unique(paths))
}

fn changed_paths_between(
    repo: &Repository,
    old_oid: Oid,
    new_oid: Oid,
) -> Result<Vec<String>, GitboxError> {
    let old_tree = repo.find_commit(old_oid)?.tree()?;
    let new_tree = repo.find_commit(new_oid)?.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;
    let mut paths = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            collect_delta_paths(delta, &mut paths);
            true
        },
        None,
        None,
        None,
    )?;
    Ok(sorted_unique(paths))
}

fn collect_delta_paths(delta: git2::DiffDelta<'_>, paths: &mut Vec<String>) {
    if let Some(path) = delta.old_file().path() {
        paths.push(repo_path_string(path));
    }
    if let Some(path) = delta.new_file().path() {
        paths.push(repo_path_string(path));
    }
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn upstream_remote_name(upstream: &str) -> Option<String> {
    upstream
        .split_once('/')
        .map(|(remote, _)| remote.trim())
        .filter(|remote| !remote.is_empty())
        .map(ToOwned::to_owned)
}

fn join_git_outputs(first: String, second: String) -> String {
    match (first.trim().is_empty(), second.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => second,
        (false, true) => first,
        (false, false) => format!("{}\n{}", first.trim_end(), second.trim_start()),
    }
}

fn clean_selected_paths(
    repo: &Repository,
    values: impl IntoIterator<Item = String>,
) -> Result<Vec<String>, GitboxError> {
    let mut cleaned = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || value == "ALL" {
            continue;
        }
        let rel = repo_path_string(&normalize_repo_path(repo, value)?);
        if !cleaned.iter().any(|item| item == &rel) {
            cleaned.push(rel);
        }
    }
    Ok(cleaned)
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

struct ConflictDisplayContext {
    current_side: &'static str,
    incoming_side: &'static str,
    source: Option<&'static str>,
}

fn default_conflict_display_context() -> ConflictDisplayContext {
    ConflictDisplayContext {
        current_side: "ours",
        incoming_side: "theirs",
        source: None,
    }
}

fn conflict_display_context(
    repo: &Repository,
    workdir: &Path,
    current: Option<&str>,
) -> Result<ConflictDisplayContext, GitboxError> {
    if is_autostash_apply_conflict(repo, workdir, current)? {
        return Ok(ConflictDisplayContext {
            current_side: "theirs",
            incoming_side: "ours",
            source: Some("autostash"),
        });
    }

    Ok(default_conflict_display_context())
}

fn is_autostash_apply_conflict(
    repo: &Repository,
    workdir: &Path,
    current: Option<&str>,
) -> Result<bool, GitboxError> {
    if git_operation_marker_exists(repo) {
        return Ok(false);
    }
    if current.is_some_and(looks_like_stash_apply_conflict) {
        return Ok(true);
    }

    let output = run_git_raw(
        workdir,
        vec!["stash".to_string(), "list".to_string(), "-1".to_string()],
        None,
    )?;
    Ok(output.success && output.stdout.to_ascii_lowercase().contains("autostash"))
}

fn git_operation_marker_exists(repo: &Repository) -> bool {
    let git_dir = repo.path();
    git_dir.join("rebase-merge").exists()
        || git_dir.join("rebase-apply").exists()
        || git_dir.join("MERGE_HEAD").exists()
        || git_dir.join("CHERRY_PICK_HEAD").exists()
        || git_dir.join("REVERT_HEAD").exists()
}

fn looks_like_stash_apply_conflict(content: &str) -> bool {
    content.contains("<<<<<<< Updated upstream") && content.contains(">>>>>>> Stashed changes")
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

struct MergeFileSimulation {
    content: String,
    conflict_count: usize,
}

fn analyze_conflict_block(block: &ConflictBlock) -> Result<ConflictBlockAnalysis, GitboxError> {
    let base = block.base.as_deref();
    let ours = block.ours.as_str();
    let theirs = block.theirs.as_str();

    if ours == theirs {
        return Ok(conflict_block_analysis(
            block.index,
            "same_change",
            "certain",
            100,
            Some("ours"),
            "两侧做出了相同修改，可以保留任意一侧。",
            Some(ours.to_string()),
        ));
    }

    if let Some(base) = base {
        if ours.is_empty() && theirs == base {
            return Ok(conflict_block_analysis(
                block.index,
                "delete_no_change",
                "certain",
                94,
                Some("ours"),
                "当前版本删除了该块，传入版本未改动，建议保留删除结果。",
                Some(String::new()),
            ));
        }
        if theirs.is_empty() && ours == base {
            return Ok(conflict_block_analysis(
                block.index,
                "delete_no_change",
                "certain",
                94,
                Some("theirs"),
                "传入版本删除了该块，当前版本未改动，建议保留删除结果。",
                Some(String::new()),
            ));
        }
        if ours == base && theirs != base {
            return Ok(conflict_block_analysis(
                block.index,
                "one_side_change",
                "certain",
                96,
                Some("theirs"),
                "只有传入版本修改了该块，建议接受传入版本。",
                Some(theirs.to_string()),
            ));
        }
        if theirs == base && ours != base {
            return Ok(conflict_block_analysis(
                block.index,
                "one_side_change",
                "certain",
                96,
                Some("ours"),
                "只有当前版本修改了该块，建议保留当前版本。",
                Some(ours.to_string()),
            ));
        }
    }

    if normalize_for_conflict_whitespace(ours) == normalize_for_conflict_whitespace(theirs) {
        return Ok(conflict_block_analysis(
            block.index,
            "whitespace_only",
            "high",
            82,
            Some("ours"),
            "两侧仅有空白字符差异，建议保留当前版本的格式。",
            Some(ours.to_string()),
        ));
    }

    if let Some(base) = base {
        let merged = simulate_merge_file(ours, base, theirs)?;
        if merged.conflict_count == 0 {
            return Ok(conflict_block_analysis(
                block.index,
                "non_overlapping",
                "high",
                78,
                None,
                "两侧修改可以通过三方文本合并合成一个无冲突结果。",
                Some(merged.content),
            ));
        }
    }

    Ok(conflict_block_analysis(
        block.index,
        "complex",
        "low",
        20,
        None,
        "两侧修改存在重叠或缺少可靠基线，需要人工判断。",
        None,
    ))
}

fn conflict_block_analysis(
    index: usize,
    kind: &str,
    confidence: &str,
    score: u8,
    suggested_side: Option<&str>,
    explanation: &str,
    replacement: Option<String>,
) -> ConflictBlockAnalysis {
    ConflictBlockAnalysis {
        index,
        kind: kind.to_string(),
        confidence: confidence.to_string(),
        score,
        suggested_side: suggested_side.map(str::to_string),
        explanation: explanation.to_string(),
        replacement,
    }
}

fn normalize_for_conflict_whitespace(content: &str) -> String {
    content
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn simulate_merge_file(
    ours: &str,
    base: &str,
    theirs: &str,
) -> Result<MergeFileSimulation, GitboxError> {
    let dir = tempfile::tempdir()?;
    let ours_path = dir.path().join("ours");
    let base_path = dir.path().join("base");
    let theirs_path = dir.path().join("theirs");
    fs::write(&ours_path, ours.as_bytes())?;
    fs::write(&base_path, base.as_bytes())?;
    fs::write(&theirs_path, theirs.as_bytes())?;
    let output = run_git_raw(
        dir.path(),
        vec![
            "merge-file".to_string(),
            "-p".to_string(),
            "--diff3".to_string(),
            "ours".to_string(),
            "base".to_string(),
            "theirs".to_string(),
        ],
        None,
    )?;
    let content = if output.stdout.is_empty() && !output.success {
        output.stderr
    } else {
        output.stdout
    };
    let conflict_count = content.matches("<<<<<<< ").count();
    Ok(MergeFileSimulation {
        content,
        conflict_count,
    })
}

fn derive_conflict_blocks_from_stages(
    base: Option<&str>,
    ours: Option<&str>,
    theirs: Option<&str>,
) -> Vec<ConflictBlock> {
    let Some(ours) = ours else {
        return Vec::new();
    };
    let Some(theirs) = theirs else {
        return Vec::new();
    };
    if ours == theirs {
        return Vec::new();
    }

    let base_lines = split_preserving_newlines(base.unwrap_or_default());
    let ours_lines = split_preserving_newlines(ours);
    let theirs_lines = split_preserving_newlines(theirs);
    let anchors = common_stage_anchors(&base_lines, &ours_lines, &theirs_lines);
    let mut blocks = Vec::new();
    let mut base_start = 0usize;
    let mut ours_start = 0usize;
    let mut theirs_start = 0usize;

    for (base_anchor, ours_anchor, theirs_anchor) in anchors.into_iter().chain(std::iter::once((
        base_lines.len(),
        ours_lines.len(),
        theirs_lines.len(),
    ))) {
        let base_part = &base_lines[base_start..base_anchor];
        let ours_part = &ours_lines[ours_start..ours_anchor];
        let theirs_part = &theirs_lines[theirs_start..theirs_anchor];
        if ours_part != theirs_part {
            blocks.push(ConflictBlock {
                index: blocks.len(),
                ours: ours_part.concat(),
                base: if base_part.is_empty() {
                    None
                } else {
                    Some(base_part.concat())
                },
                theirs: theirs_part.concat(),
            });
        }

        base_start = base_anchor.saturating_add(1);
        ours_start = ours_anchor.saturating_add(1);
        theirs_start = theirs_anchor.saturating_add(1);
    }

    blocks
}

fn common_stage_anchors(
    base_lines: &[String],
    ours_lines: &[String],
    theirs_lines: &[String],
) -> Vec<(usize, usize, usize)> {
    let mut anchors = Vec::new();
    let mut ours_from = 0usize;
    let mut theirs_from = 0usize;

    for (base_index, line) in base_lines.iter().enumerate() {
        let Some(ours_index) = find_line_after(ours_lines, line, ours_from) else {
            continue;
        };
        let Some(theirs_index) = find_line_after(theirs_lines, line, theirs_from) else {
            continue;
        };
        anchors.push((base_index, ours_index, theirs_index));
        ours_from = ours_index + 1;
        theirs_from = theirs_index + 1;
    }

    anchors
}

fn find_line_after(lines: &[String], needle: &str, start: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, line)| (line == needle).then_some(index))
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
    let value = path.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        strip_windows_verbatim_prefix(&value)
    }
    #[cfg(not(windows))]
    {
        value
    }
}

#[cfg(any(windows, test))]
fn strip_windows_verbatim_prefix(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{rest}");
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return rest.to_string();
    }
    value.to_string()
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
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
        let state = operation_state_core(path)?;
        if !state.conflicted_paths.is_empty() {
            return Ok(CommandResult {
                ok: false,
                message: conflict_message,
                output: output.combined_output(),
            });
        }
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
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
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

fn read_treeish_file_text(
    repo: &Repository,
    treeish: &str,
    pathspec: &str,
) -> Result<Option<String>, GitboxError> {
    let tree = match repo
        .revparse_single(treeish)
        .ok()
        .and_then(|object| object.peel_to_tree().ok())
    {
        Some(tree) => tree,
        None => return Ok(None),
    };
    let entry = match tree.get_path(Path::new(pathspec)) {
        Ok(entry) => entry,
        Err(_) => return Ok(None),
    };
    let object = entry.to_object(repo)?;
    let Some(blob) = object.as_blob() else {
        return Ok(None);
    };
    Ok(bytes_to_preview_text(blob.content()))
}

fn treeish_file_exists(
    repo: &Repository,
    treeish: &str,
    pathspec: &str,
) -> Result<bool, GitboxError> {
    let tree = match repo
        .revparse_single(treeish)
        .ok()
        .and_then(|object| object.peel_to_tree().ok())
    {
        Some(tree) => tree,
        None => return Ok(false),
    };
    Ok(tree.get_path(Path::new(pathspec)).is_ok())
}

fn read_index_file_text(repo: &Repository, pathspec: &str) -> Result<Option<String>, GitboxError> {
    let index = repo.index()?;
    let Some(entry) = index.get_path(Path::new(pathspec), 0) else {
        return Ok(None);
    };
    let blob = repo.find_blob(entry.id)?;
    Ok(bytes_to_preview_text(blob.content()))
}

fn read_workdir_file_text(
    repo: &Repository,
    pathspec: &str,
) -> Result<Option<String>, GitboxError> {
    let path = repo_workdir(repo)?.join(pathspec);
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.is_dir() || metadata.len() > 2_000_000 {
        return Ok(None);
    }

    let bytes = fs::read(path)?;
    Ok(bytes_to_preview_text(&bytes))
}

fn bytes_to_preview_text(bytes: &[u8]) -> Option<String> {
    if bytes.len() > 2_000_000 || bytes.contains(&0) {
        return None;
    }
    std::str::from_utf8(bytes).map(ToOwned::to_owned).ok()
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
