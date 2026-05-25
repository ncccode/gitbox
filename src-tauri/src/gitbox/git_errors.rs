fn translate_git_failure_message(message: &str) -> Option<String> {
    let lower = message.to_ascii_lowercase();
    if !lower.contains("untracked working tree files would be overwritten") {
        return None;
    }

    let action = if lower.contains("would be overwritten by merge") {
        "拉取或合并"
    } else if lower.contains("would be overwritten by checkout") {
        "切换分支"
    } else {
        "Git 操作"
    };
    let files = extract_overwritten_untracked_files(message);
    let mut translated = format!(
        "本地未跟踪文件会被本次{action}覆盖，Git 已中止以保护这些文件。请先移动、删除、暂存或提交这些文件后再重试。"
    );

    if !files.is_empty() {
        translated.push_str("\n\n受影响文件：");
        for file in files {
            translated.push_str("\n- ");
            translated.push_str(&file);
        }
    }

    Some(translated)
}

fn extract_overwritten_untracked_files(message: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut capturing = false;

    for raw_line in message.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line
            .to_ascii_lowercase()
            .contains("untracked working tree files would be overwritten")
        {
            capturing = true;
            continue;
        }
        if !capturing {
            continue;
        }
        if line.starts_with("Please ")
            || line.starts_with("Aborting")
            || line.starts_with("Created autostash")
            || line.starts_with("Applied autostash")
        {
            break;
        }

        files.push(line.to_string());
    }

    sorted_unique(files)
}
