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

pub fn pull_preflight_core(
    path: &str,
    remote_name: Option<String>,
) -> Result<PullPreflight, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let (remote, target) = current_pull_remote_and_target(&repo, remote_name)?;
    run_git(&workdir, vec!["fetch".to_string(), remote.clone()], None)?;

    let repo = Repository::discover(path)?;
    build_pull_preflight(&repo, remote, target)
}

pub fn pull_core(
    path: &str,
    remote_name: Option<String>,
    smart_merge: bool,
) -> Result<CommandResult, GitboxError> {
    let repo = Repository::discover(path)?;
    let workdir = repo_workdir(&repo)?;
    let (remote, target) = current_pull_remote_and_target(&repo, remote_name)?;
    let fetch_output = run_git(&workdir, vec!["fetch".to_string(), remote.clone()], None)?;

    let repo = Repository::discover(path)?;
    let preflight = build_pull_preflight(&repo, remote, target)?;
    let head_oid = preflight
        .head
        .as_deref()
        .ok_or_else(|| GitboxError::Message("当前 HEAD 无法拉取".to_string()))
        .and_then(|oid| Oid::from_str(oid).map_err(GitboxError::Git))?;
    let target_oid = preflight
        .target_oid
        .as_deref()
        .ok_or_else(|| GitboxError::Message(format!("找不到远程分支 {}", preflight.target)))
        .and_then(|oid| Oid::from_str(oid).map_err(GitboxError::Git))?;

    if preflight.needs_confirmation && !smart_merge {
        return Err(GitboxError::Message(format!(
            "本地未提交修改与 {} 的更新重叠，请确认后使用智能合并",
            preflight.target
        )));
    }

    if head_oid == target_oid {
        return Ok(CommandResult {
            ok: true,
            message: format!("当前分支已是 {} 最新状态", preflight.target),
            output: fetch_output,
        });
    }

    let merge_base = merge_base_or_none(&repo, head_oid, target_oid)?.ok_or_else(|| {
        GitboxError::Message(unrelated_histories_pull_message(
            &preflight.target,
            &preflight.remote,
        ))
    })?;
    let use_autostash = smart_merge && !preflight.overlapping_paths.is_empty();
    if merge_base == head_oid {
        let mut args = vec!["merge".to_string(), "--ff-only".to_string()];
        if use_autostash {
            args.push("--autostash".to_string());
        }
        args.push(preflight.target.clone());
        let mut result = run_git_operation(
            path,
            args,
            format!("已快进拉取 {}", preflight.target),
            format!(
                "已拉取 {}，智能合并本地修改时产生冲突，请在三栏合并窗口解决",
                preflight.target
            ),
        )?;
        result.output = join_git_outputs(fetch_output, result.output);
        return Ok(CommandResult {
            ok: result.ok,
            message: result.message,
            output: result.output,
        });
    }

    if merge_base == target_oid {
        return Ok(CommandResult {
            ok: true,
            message: format!("当前分支已包含 {} 的更新", preflight.target),
            output: fetch_output,
        });
    }

    let mut args = vec!["merge".to_string(), "--no-edit".to_string()];
    if use_autostash {
        args.push("--autostash".to_string());
    }
    args.push(preflight.target.clone());
    let mut result = run_git_operation(
        path,
        args,
        format!("已拉取并合并 {}", preflight.target),
        format!(
            "拉取 {} 时需要合并，请在三栏合并窗口解决冲突后继续",
            preflight.target
        ),
    )?;
    result.output = join_git_outputs(fetch_output, result.output);
    Ok(result)
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
        .ok_or_else(|| GitboxError::Message("当前 HEAD 不是本地分支，不能推送".to_string()))?
        .to_string();
    let head_oid = head
        .target()
        .ok_or_else(|| GitboxError::Message("当前 HEAD 无法推送".to_string()))?;
    let target = target_branch
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| branch.clone());
    ensure_no_pending_operation_for_action(path, "推送")?;
    ensure_outgoing_diff_has_no_conflict_markers(&repo, &workdir, &name, &target)?;

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
    update_remote_tracking_ref_after_push(&repo, &name, &target, head_oid)?;

    Ok(CommandResult {
        ok: true,
        message: format!("已推送到 {name}/{target}"),
        output,
    })
}

fn update_remote_tracking_ref_after_push(
    repo: &Repository,
    remote: &str,
    target_branch: &str,
    oid: Oid,
) -> Result<(), GitboxError> {
    repo.reference(
        &format!("refs/remotes/{remote}/{target_branch}"),
        oid,
        true,
        "GitBox: update remote-tracking ref after successful push",
    )?;
    Ok(())
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
