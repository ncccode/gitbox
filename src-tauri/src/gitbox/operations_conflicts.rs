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

pub fn preview_merge_core(path: &str, target: String) -> Result<MergePreview, GitboxError> {
    let target = clean_ref_input(target, "请选择要预览合并的分支")?;
    let repo = Repository::discover(path)?;
    let head_oid = repo
        .head()?
        .target()
        .ok_or_else(|| GitboxError::Message("当前 HEAD 无法预览合并".to_string()))?;
    let target_oid = repo
        .revparse_single(&target)
        .and_then(|object| object.peel_to_commit())
        .map(|commit| commit.id())
        .map_err(|_| GitboxError::Message(format!("找不到合并目标 {target}")))?;
    let merge_base = merge_base_or_none(&repo, head_oid, target_oid)?;
    let Some(merge_base) = merge_base else {
        return Ok(MergePreview {
            target,
            head: Some(head_oid.to_string()),
            target_oid: Some(target_oid.to_string()),
            merge_base: None,
            clean: false,
            files: Vec::new(),
            summary: MergePreviewSummary {
                manual: 1,
                ..MergePreviewSummary::default()
            },
        });
    };

    let local_paths = changed_paths_between(&repo, merge_base, head_oid)?;
    let incoming_paths = changed_paths_between(&repo, merge_base, target_oid)?;
    let local_set = local_paths.iter().cloned().collect::<HashSet<_>>();
    let incoming_set = incoming_paths.iter().cloned().collect::<HashSet<_>>();
    let mut paths = sorted_unique(local_paths.into_iter().chain(incoming_paths));
    let base_ref = merge_base.to_string();
    let head_ref = head_oid.to_string();
    let target_ref = target_oid.to_string();
    let mut files = Vec::new();
    let mut summary = MergePreviewSummary::default();

    for pathspec in paths.drain(..) {
        let local_changed = local_set.contains(&pathspec);
        let incoming_changed = incoming_set.contains(&pathspec);
        if !(local_changed && incoming_changed) {
            summary.clean += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "clean".to_string(),
                conflict_count: 0,
                auto_resolvable: true,
                explanation: "只有一侧修改，Git 可直接合并。".to_string(),
            });
            continue;
        }

        let base_exists = treeish_file_exists(&repo, &base_ref, &pathspec)?;
        let ours_exists = treeish_file_exists(&repo, &head_ref, &pathspec)?;
        let theirs_exists = treeish_file_exists(&repo, &target_ref, &pathspec)?;
        let base_text = read_treeish_file_text(&repo, &base_ref, &pathspec)?;
        let ours_text = read_treeish_file_text(&repo, &head_ref, &pathspec)?;
        let theirs_text = read_treeish_file_text(&repo, &target_ref, &pathspec)?;

        if (base_exists && base_text.is_none())
            || (ours_exists && ours_text.is_none())
            || (theirs_exists && theirs_text.is_none())
        {
            summary.binary += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "binary".to_string(),
                conflict_count: 0,
                auto_resolvable: false,
                explanation: "包含二进制或过大的文件版本，需要人工检查。".to_string(),
            });
            continue;
        }

        if base_exists && (ours_exists != theirs_exists) {
            summary.add_delete += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "add_delete".to_string(),
                conflict_count: 1,
                auto_resolvable: false,
                explanation: "一侧删除文件，另一侧仍在修改，需要人工选择保留或删除。".to_string(),
            });
            continue;
        }

        let base = base_text.unwrap_or_default();
        let ours = ours_text.unwrap_or_default();
        let theirs = theirs_text.unwrap_or_default();
        if ours == theirs {
            summary.clean += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "clean".to_string(),
                conflict_count: 0,
                auto_resolvable: true,
                explanation: "两侧最终内容一致。".to_string(),
            });
            continue;
        }

        let merged = simulate_merge_file(&ours, &base, &theirs)?;
        if merged.conflict_count == 0 {
            summary.auto_resolvable += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "auto_resolvable".to_string(),
                conflict_count: 0,
                auto_resolvable: true,
                explanation: "三方文本合并可生成无冲突结果。".to_string(),
            });
        } else {
            summary.manual += 1;
            files.push(MergePreviewFile {
                path: pathspec,
                category: "manual".to_string(),
                conflict_count: merged.conflict_count,
                auto_resolvable: false,
                explanation: format!(
                    "预计产生 {} 个文本冲突，需要进入合并编辑器。",
                    merged.conflict_count
                ),
            });
        }
    }

    Ok(MergePreview {
        target,
        head: Some(head_oid.to_string()),
        target_oid: Some(target_oid.to_string()),
        merge_base: Some(merge_base.to_string()),
        clean: summary.manual == 0 && summary.add_delete == 0 && summary.binary == 0,
        files,
        summary,
    })
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
    let pushed_oid = repo
        .revparse_single(&oid)?
        .peel_to_commit()
        .map(|commit| commit.id())?;
    let output = run_git(
        &workdir,
        vec![
            "push".to_string(),
            remote.clone(),
            format!("{oid}:refs/heads/{target}"),
        ],
        None,
    )?;
    update_remote_tracking_ref_after_push(&repo, &remote, &target, pushed_oid)?;
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

    if action == "continue" {
        let repo = Repository::discover(path)?;
        ensure_index_ready_for_commit(&repo, None, "继续当前操作")?;
    }

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
    let base = read_index_stage(&workdir, 1, &pathspec)?;
    let ours = read_index_stage(&workdir, 2, &pathspec)?;
    let theirs = read_index_stage(&workdir, 3, &pathspec)?;
    let display_context = conflict_display_context(&repo, &workdir, current.as_deref())?;
    let mut blocks = current
        .as_deref()
        .map(parse_conflict_blocks_for_response)
        .unwrap_or_default();
    if blocks.is_empty() {
        blocks =
            derive_conflict_blocks_from_stages(base.as_deref(), ours.as_deref(), theirs.as_deref());
    }

    Ok(ConflictDetails {
        path: pathspec.clone(),
        base,
        ours,
        theirs,
        current,
        current_side: display_context.current_side.to_string(),
        incoming_side: display_context.incoming_side.to_string(),
        conflict_source: display_context.source.map(str::to_string),
        blocks,
    })
}

pub fn analyze_conflict_file_core(
    path: &str,
    file_path: String,
) -> Result<ConflictAnalysis, GitboxError> {
    let details = conflict_details_core(path, file_path)?;
    let blocks = details
        .blocks
        .iter()
        .map(analyze_conflict_block)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ConflictAnalysis {
        path: details.path,
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
    let marker_lines = conflict_marker_lines_from_content(&pathspec, &current);
    if !marker_lines.is_empty() {
        return Err(conflict_marker_error("标记为已解决", &marker_lines));
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
        let marker_lines = conflict_marker_lines_from_content(&pathspec, &content);
        if !marker_lines.is_empty() {
            return Err(conflict_marker_error("标记为已解决", &marker_lines));
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
