pub fn open_repo_core(path: &str) -> Result<RepositoryInfo, GitboxError> {
    let repo = Repository::discover(path)?;
    repository_info(&repo)
}

pub fn filter_project_directories_core(paths: Vec<String>) -> Result<Vec<String>, GitboxError> {
    let mut directories = Vec::new();
    let mut seen = HashSet::new();

    for raw_path in paths {
        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            continue;
        }

        let path = PathBuf::from(trimmed);
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }

        let normalized = fs::canonicalize(&path).unwrap_or(path);
        let value = path_string(&normalized);
        if seen.insert(value.clone()) {
            directories.push(value);
        }
    }

    Ok(directories)
}

pub fn open_project_terminal_core(path: String) -> Result<CommandResult, GitboxError> {
    let path = PathBuf::from(clean_ref_input(path, "请输入项目目录")?);
    let path = fs::canonicalize(path)?;
    if !fs::metadata(&path)?.is_dir() {
        return Err(GitboxError::Message("项目路径不是文件夹".to_string()));
    }

    open_system_terminal(&path)?;
    Ok(CommandResult {
        ok: true,
        message: "已在系统终端打开项目目录".to_string(),
        output: path_string(&path),
    })
}

pub fn open_project_directory_core(path: String) -> Result<CommandResult, GitboxError> {
    let path = PathBuf::from(clean_ref_input(path, "请输入项目目录")?);
    let path = fs::canonicalize(path)?;
    if !fs::metadata(&path)?.is_dir() {
        return Err(GitboxError::Message("项目路径不是文件夹".to_string()));
    }

    open_system_file_manager(&path)?;
    Ok(CommandResult {
        ok: true,
        message: "已在系统文件管理器打开项目目录".to_string(),
        output: path_string(&path),
    })
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
