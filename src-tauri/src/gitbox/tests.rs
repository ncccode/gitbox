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

    fn conflict_block(index: usize, base: Option<&str>, ours: &str, theirs: &str) -> ConflictBlock {
        ConflictBlock {
            index,
            ours: ours.to_string(),
            base: base.map(ToOwned::to_owned),
            theirs: theirs.to_string(),
        }
    }

    #[test]
    fn path_string_hides_windows_verbatim_prefixes() {
        assert_eq!(
            strip_windows_verbatim_prefix("\\\\?\\D:\\project\\repo"),
            "D:\\project\\repo"
        );
        assert_eq!(
            strip_windows_verbatim_prefix("\\\\?\\UNC\\server\\share\\repo"),
            "\\\\server\\share\\repo"
        );
        assert_eq!(
            strip_windows_verbatim_prefix("D:\\project\\repo"),
            "D:\\project\\repo"
        );
    }

    #[test]
    fn filter_project_directories_keeps_only_existing_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let project = dir.path().join("project");
        fs::create_dir_all(&project).expect("create project");
        let file = dir.path().join("note.txt");
        fs::write(&file, "not a directory").expect("write file");

        let paths = filter_project_directories_core(vec![
            file.to_string_lossy().to_string(),
            project.to_string_lossy().to_string(),
            format!(" {} ", project.to_string_lossy()),
            dir.path().join("missing").to_string_lossy().to_string(),
            String::new(),
        ])
        .expect("filter project directories");

        assert_eq!(
            paths,
            vec![path_string(&fs::canonicalize(project).unwrap())]
        );
    }

    #[test]
    fn git_failure_message_translates_untracked_overwrite_error() {
        let output = GitProcessOutput {
            success: false,
            stdout: "Updating 7dbcd7d..441c9cc\nCreated autostash: 99b70bc\n".to_string(),
            stderr: "error: The following untracked working tree files would be overwritten by merge:\n\tdocs/colleague-ops-runbook.md\n\tpages/colleague-ops.html\nPlease move or remove them before you merge.\nAborting\nApplied autostash.\n".to_string(),
        };

        let message = output.failure_message();
        assert!(message.contains("本地未跟踪文件会被本次拉取或合并覆盖"));
        assert!(message.contains("docs/colleague-ops-runbook.md"));
        assert!(message.contains("pages/colleague-ops.html"));
        assert!(!message.contains("The following untracked working tree files"));
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
    fn repo_status_expands_untracked_directory_files() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "vendor/autoload.php", "<?php\n");
        write_file(dir.path(), "vendor/composer/ClassLoader.php", "<?php\n");

        let status = repo_status_core(dir.path().to_str().unwrap(), false).expect("status");
        assert_eq!(status.counts.untracked, 2);
        assert_eq!(
            status
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["vendor/autoload.php", "vendor/composer/ClassLoader.php"]
        );
        assert!(status.files.iter().all(|file| file.untracked));
    }

    #[test]
    fn commit_can_include_worktree_changes_when_requested() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "src/main.rs", "fn main() {}\n");

        let commit = commit_with_full_options_and_worktree_core(
            dir.path().to_str().unwrap(),
            "add main directly".to_string(),
            false,
            false,
            false,
            None,
            true,
        )
        .expect("commit worktree");

        assert_eq!(commit.oid.len(), 40);
        assert!(
            repo_status_core(dir.path().to_str().unwrap(), false)
                .expect("status")
                .branch
                .clean
        );
    }

    #[test]
    fn commit_selected_paths_preserves_unchecked_staged_changes() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        write_file(dir.path(), "a.txt", "base a\n");
        write_file(dir.path(), "b.txt", "base b\n");
        stage_paths_core(path, vec!["a.txt".to_string(), "b.txt".to_string()]).expect("stage base");
        commit_core(path, "base".to_string()).expect("commit base");

        write_file(dir.path(), "a.txt", "selected a\n");
        write_file(dir.path(), "b.txt", "unchecked b\n");
        stage_paths_core(path, vec!["a.txt".to_string(), "b.txt".to_string()]).expect("stage both");

        let commit = commit_with_full_options_and_selection_core(
            path,
            "selected only".to_string(),
            false,
            false,
            false,
            None,
            false,
            Some(vec!["a.txt".to_string()]),
        )
        .expect("commit selected");

        let details = commit_details_core(path, commit.oid).expect("details");
        assert_eq!(details.files.len(), 1);
        assert_eq!(details.files[0].path, "a.txt");

        let status = repo_status_core(path, false).expect("status");
        assert_eq!(status.counts.staged, 1);
        assert!(status
            .files
            .iter()
            .any(|file| file.path == "b.txt" && file.staged));
    }

    #[test]
    fn commit_selected_paths_can_commit_untracked_worktree_file() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        initial_commit(dir.path());
        write_file(dir.path(), "selected.txt", "selected\n");
        write_file(dir.path(), "unchecked.txt", "unchecked\n");

        let commit = commit_with_full_options_and_selection_core(
            path,
            "selected untracked".to_string(),
            false,
            false,
            false,
            None,
            false,
            Some(vec!["selected.txt".to_string()]),
        )
        .expect("commit selected untracked");

        let details = commit_details_core(path, commit.oid).expect("details");
        assert_eq!(details.files.len(), 1);
        assert_eq!(details.files[0].path, "selected.txt");

        let status = repo_status_core(path, false).expect("status");
        assert_eq!(status.counts.untracked, 1);
        assert!(status
            .files
            .iter()
            .any(|file| file.path == "unchecked.txt" && file.untracked));
    }

    #[test]
    fn commit_selected_paths_finishes_merge_without_partial_commit() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        write_file(dir.path(), "conflict-demo.txt", "base\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage base");
        commit_core(path, "base".to_string()).expect("commit base");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "-B".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main");
        run_git(
            dir.path(),
            vec![
                "checkout".to_string(),
                "-b".to_string(),
                "feature".to_string(),
            ],
            None,
        )
        .expect("checkout feature");
        write_file(dir.path(), "conflict-demo.txt", "feature\n");
        write_file(dir.path(), "colleague-long-path.txt", "feature side\n");
        stage_paths_core(
            path,
            vec![
                "conflict-demo.txt".to_string(),
                "colleague-long-path.txt".to_string(),
            ],
        )
        .expect("stage feature");
        commit_core(path, "feature change".to_string()).expect("commit feature");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main again");
        write_file(dir.path(), "conflict-demo.txt", "main\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage main");
        commit_core(path, "main change".to_string()).expect("commit main");

        let merge = run_git_raw(
            dir.path(),
            vec!["merge".to_string(), "feature".to_string()],
            None,
        )
        .expect("merge process");
        assert!(!merge.success);
        assert!(Repository::open(dir.path())
            .unwrap()
            .path()
            .join("MERGE_HEAD")
            .exists());

        write_file(dir.path(), "conflict-demo.txt", "resolved\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage resolution");

        let commit = commit_with_full_options_and_selection_core(
            path,
            "merge feature".to_string(),
            false,
            false,
            false,
            None,
            false,
            Some(vec!["conflict-demo.txt".to_string()]),
        )
        .expect("commit merge");

        let repo = Repository::open(dir.path()).expect("repo");
        let head = repo.head().expect("head").peel_to_commit().expect("commit");
        assert_eq!(head.id().to_string(), commit.oid);
        assert_eq!(head.parent_count(), 2);
        assert!(!repo.path().join("MERGE_HEAD").exists());
        assert_eq!(
            fs::read_to_string(dir.path().join("colleague-long-path.txt")).unwrap(),
            "feature side\n"
        );
        assert!(repo_status_core(path, false).expect("status").branch.clean);
    }

    #[test]
    fn commit_merge_rejects_staged_conflict_markers() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        write_file(dir.path(), "conflict-demo.txt", "base\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage base");
        commit_core(path, "base".to_string()).expect("commit base");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "-B".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main");
        run_git(
            dir.path(),
            vec![
                "checkout".to_string(),
                "-b".to_string(),
                "feature".to_string(),
            ],
            None,
        )
        .expect("checkout feature");
        write_file(dir.path(), "conflict-demo.txt", "feature\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage feature");
        commit_core(path, "feature change".to_string()).expect("commit feature");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main again");
        write_file(dir.path(), "conflict-demo.txt", "main\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage main");
        commit_core(path, "main change".to_string()).expect("commit main");
        let head_before = repo_head_oid(path).expect("head before merge");

        let merge = run_git_raw(
            dir.path(),
            vec!["merge".to_string(), "feature".to_string()],
            None,
        )
        .expect("merge process");
        assert!(!merge.success);

        write_file(
            dir.path(),
            "conflict-demo.txt",
            "<<<<<<< HEAD\nmain\n=======\nfeature\n>>>>>>> feature\n",
        );
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()])
            .expect("stage bad resolution");

        let err = commit_with_full_options_and_selection_core(
            path,
            "merge feature".to_string(),
            false,
            false,
            false,
            None,
            false,
            Some(vec!["conflict-demo.txt".to_string()]),
        )
        .expect_err("reject conflict markers");

        let message = err.to_string();
        assert!(message.contains("冲突标记"));
        assert!(message.contains("conflict-demo.txt"));
        assert_eq!(
            repo_head_oid(path).expect("head after rejection"),
            head_before
        );
        assert!(Repository::open(dir.path())
            .unwrap()
            .path()
            .join("MERGE_HEAD")
            .exists());
    }

    #[test]
    fn commit_merge_rejection_preserves_conflict_editor_stages() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        write_file(dir.path(), "conflict-demo.txt", "base\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage base");
        commit_core(path, "base".to_string()).expect("commit base");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "-B".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main");
        run_git(
            dir.path(),
            vec![
                "checkout".to_string(),
                "-b".to_string(),
                "feature".to_string(),
            ],
            None,
        )
        .expect("checkout feature");
        write_file(dir.path(), "conflict-demo.txt", "feature\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage feature");
        commit_core(path, "feature change".to_string()).expect("commit feature");

        run_git(
            dir.path(),
            vec!["checkout".to_string(), "main".to_string()],
            None,
        )
        .expect("checkout main again");
        write_file(dir.path(), "conflict-demo.txt", "main\n");
        stage_paths_core(path, vec!["conflict-demo.txt".to_string()]).expect("stage main");
        commit_core(path, "main change".to_string()).expect("commit main");

        let merge = run_git_raw(
            dir.path(),
            vec!["merge".to_string(), "feature".to_string()],
            None,
        )
        .expect("merge process");
        assert!(!merge.success);

        let err = commit_with_full_options_and_selection_core(
            path,
            "merge feature".to_string(),
            false,
            false,
            false,
            None,
            false,
            Some(vec!["conflict-demo.txt".to_string()]),
        )
        .expect_err("reject conflict markers before staging");

        let message = err.to_string();
        assert!(message.contains("冲突标记"));
        assert!(message.contains("conflict-demo.txt"));

        let status = repo_status_core(path, false).expect("status");
        assert_eq!(status.counts.conflicted, 1);
        assert!(status
            .files
            .iter()
            .any(|file| file.path == "conflict-demo.txt" && file.conflicted));

        let details =
            conflict_details_core(path, "conflict-demo.txt".to_string()).expect("details");
        assert!(details.base.is_some());
        assert!(details.ours.is_some());
        assert!(details.theirs.is_some());
        assert!(!details.blocks.is_empty());
    }

    #[test]
    fn push_rejects_outgoing_commit_with_conflict_markers() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        initial_commit(dir.path());

        let remote_dir = tempfile::tempdir().expect("remote dir");
        Repository::init_bare(remote_dir.path()).expect("bare remote");
        run_git(
            dir.path(),
            vec![
                "remote".to_string(),
                "add".to_string(),
                "origin".to_string(),
                remote_dir.path().to_string_lossy().to_string(),
            ],
            None,
        )
        .expect("add remote");
        run_git(
            dir.path(),
            vec![
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                "HEAD".to_string(),
            ],
            None,
        )
        .expect("push base");

        write_file(
            dir.path(),
            "docs/remote-handoff.md",
            "# Remote Handoff\n\n<<<<<<< HEAD\nlocal\n=======\nremote\n>>>>>>> origin/main\n",
        );
        run_git(
            dir.path(),
            vec!["add".to_string(), "docs/remote-handoff.md".to_string()],
            None,
        )
        .expect("stage bad commit");
        run_git(
            dir.path(),
            vec![
                "commit".to_string(),
                "-m".to_string(),
                "bad conflict result".to_string(),
            ],
            None,
        )
        .expect("create bad commit outside GitBox");

        let err =
            push_with_options_core(path, Some("origin".to_string()), None, false, false, false)
                .expect_err("reject push");
        let message = err.to_string();
        assert!(message.contains("冲突标记"));
        assert!(message.contains("docs/remote-handoff.md"));
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
    fn push_with_options_refreshes_remote_tracking_ref() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        add_remote_core(path, "origin".to_string(), remote_url).expect("add local remote");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            true,
            false,
            false,
        )
        .expect("push initial");

        write_file(dir.path(), "unpushed.txt", "local ahead\n");
        stage_paths_core(path, vec!["unpushed.txt".to_string()]).expect("stage");
        commit_core(path, "local ahead".to_string()).expect("commit");
        let before_push = branch_summary_core(path, false).expect("summary before push");
        assert_eq!(before_push.ahead, 1);

        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            false,
            false,
            false,
        )
        .expect("push");

        let after_push = branch_summary_core(path, false).expect("summary after push");
        assert_eq!(after_push.ahead, 0);
        assert_eq!(after_push.behind, 0);

        let repo = Repository::discover(dir.path()).expect("repo");
        let head_oid = repo.head().expect("head").target().expect("head oid");
        let remote_oid = repo
            .find_reference(&format!("refs/remotes/origin/{branch}"))
            .expect("remote tracking ref")
            .target()
            .expect("remote tracking oid");
        assert_eq!(remote_oid, head_oid);
    }

    #[test]
    fn pull_diverged_branch_starts_merge_conflict_flow() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        add_remote_core(path, "origin".to_string(), remote_url.clone()).expect("add local remote");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            true,
            false,
            false,
        )
        .expect("push initial");

        let clone_parent = tempfile::tempdir().expect("clone parent");
        run_git(
            clone_parent.path(),
            vec!["clone".to_string(), remote_url, "remote-work".to_string()],
            None,
        )
        .expect("clone remote");
        let remote_workdir = clone_parent.path().join("remote-work");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.name".to_string(),
                "remote".to_string(),
            ],
            None,
        )
        .expect("remote user name");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.email".to_string(),
                "remote@example.test".to_string(),
            ],
            None,
        )
        .expect("remote user email");
        write_file(&remote_workdir, "README.md", "remote change\n");
        run_git(
            &remote_workdir,
            vec!["add".to_string(), "README.md".to_string()],
            None,
        )
        .expect("stage remote");
        run_git(
            &remote_workdir,
            vec!["commit".to_string(), "-m".to_string(), "remote".to_string()],
            None,
        )
        .expect("commit remote");
        run_git(&remote_workdir, vec!["push".to_string()], None).expect("push remote");

        write_file(dir.path(), "README.md", "local change\n");
        stage_paths_core(path, vec!["README.md".to_string()]).expect("stage local");
        commit_core(path, "local".to_string()).expect("commit local");

        let result = pull_core(path, Some("origin".to_string()), false).expect("pull starts merge");
        assert!(!result.ok);
        assert!(result.message.contains("三栏合并窗口"));
        assert!(!result.output.contains("Not possible to fast-forward"));

        let state = operation_state_core(path).expect("operation state");
        assert_eq!(state.operation.as_deref(), Some("merge"));
        assert_eq!(state.conflicted_paths, vec!["README.md".to_string()]);

        let details =
            conflict_details_core(path, "README.md".to_string()).expect("conflict details");
        assert!(details
            .ours
            .as_deref()
            .unwrap_or_default()
            .contains("local change"));
        assert!(details
            .theirs
            .as_deref()
            .unwrap_or_default()
            .contains("remote change"));
        assert_eq!(details.current_side, "ours");
        assert_eq!(details.incoming_side, "theirs");
        assert_eq!(details.conflict_source, None);
    }

    #[test]
    fn pull_preflight_detects_uncommitted_remote_overlap() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        add_remote_core(path, "origin".to_string(), remote_url.clone()).expect("add local remote");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            true,
            false,
            false,
        )
        .expect("push initial");

        let clone_parent = tempfile::tempdir().expect("clone parent");
        run_git(
            clone_parent.path(),
            vec!["clone".to_string(), remote_url, "remote-work".to_string()],
            None,
        )
        .expect("clone remote");
        let remote_workdir = clone_parent.path().join("remote-work");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.name".to_string(),
                "remote".to_string(),
            ],
            None,
        )
        .expect("remote user name");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.email".to_string(),
                "remote@example.test".to_string(),
            ],
            None,
        )
        .expect("remote user email");
        write_file(&remote_workdir, "README.md", "remote change\n");
        run_git(
            &remote_workdir,
            vec!["add".to_string(), "README.md".to_string()],
            None,
        )
        .expect("stage remote");
        run_git(
            &remote_workdir,
            vec!["commit".to_string(), "-m".to_string(), "remote".to_string()],
            None,
        )
        .expect("commit remote");
        run_git(&remote_workdir, vec!["push".to_string()], None).expect("push remote");

        write_file(dir.path(), "README.md", "local draft\n");

        let preview =
            pull_preflight_core(path, Some("origin".to_string())).expect("pull preflight");
        assert!(preview.needs_confirmation);
        assert!(preview.fast_forward);
        assert_eq!(preview.overlapping_paths, vec!["README.md".to_string()]);
        assert!(preview
            .remote_changed_paths
            .contains(&"README.md".to_string()));
        assert!(preview
            .local_changed_paths
            .contains(&"README.md".to_string()));
    }

    #[test]
    fn pull_preflight_reports_unrelated_histories_clearly() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        let remote_repo = Repository::init(remote_dir.path()).expect("init remote repo");
        {
            let mut config = remote_repo.config().expect("remote config");
            config
                .set_str("user.name", "remote")
                .expect("remote user name");
            config
                .set_str("user.email", "remote@example.test")
                .expect("remote user email");
        }
        run_git(
            remote_dir.path(),
            vec![
                "symbolic-ref".to_string(),
                "HEAD".to_string(),
                format!("refs/heads/{branch}"),
            ],
            None,
        )
        .expect("set remote head");
        write_file(remote_dir.path(), "README.md", "remote root\n");
        run_git(
            remote_dir.path(),
            vec!["add".to_string(), "README.md".to_string()],
            None,
        )
        .expect("stage remote root");
        run_git(
            remote_dir.path(),
            vec![
                "commit".to_string(),
                "-m".to_string(),
                "remote root".to_string(),
            ],
            None,
        )
        .expect("commit remote root");

        add_remote_core(
            path,
            "origin".to_string(),
            remote_dir.path().to_string_lossy().to_string(),
        )
        .expect("add unrelated remote");

        let error = pull_preflight_core(path, Some("origin".to_string()))
            .expect_err("unrelated histories should be reported");
        let message = error.to_string();
        assert!(message.contains("没有共同提交历史"));
        assert!(!message.contains("class=Merge"));
    }

    #[test]
    fn smart_pull_uses_three_way_conflict_for_dirty_overlap() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let remote_dir = tempfile::tempdir().expect("remote tempdir");
        Repository::init_bare(remote_dir.path()).expect("init bare remote");
        let remote_url = remote_dir.path().to_string_lossy().to_string();
        let path = dir.path().to_str().unwrap();
        let branch = branch_summary_core(path, false)
            .expect("summary")
            .current_branch
            .expect("current branch");

        add_remote_core(path, "origin".to_string(), remote_url.clone()).expect("add local remote");
        push_with_options_core(
            path,
            Some("origin".to_string()),
            Some(branch.clone()),
            true,
            false,
            false,
        )
        .expect("push initial");

        let clone_parent = tempfile::tempdir().expect("clone parent");
        run_git(
            clone_parent.path(),
            vec!["clone".to_string(), remote_url, "remote-work".to_string()],
            None,
        )
        .expect("clone remote");
        let remote_workdir = clone_parent.path().join("remote-work");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.name".to_string(),
                "remote".to_string(),
            ],
            None,
        )
        .expect("remote user name");
        run_git(
            &remote_workdir,
            vec![
                "config".to_string(),
                "user.email".to_string(),
                "remote@example.test".to_string(),
            ],
            None,
        )
        .expect("remote user email");
        write_file(&remote_workdir, "README.md", "remote change\n");
        run_git(
            &remote_workdir,
            vec!["add".to_string(), "README.md".to_string()],
            None,
        )
        .expect("stage remote");
        run_git(
            &remote_workdir,
            vec!["commit".to_string(), "-m".to_string(), "remote".to_string()],
            None,
        )
        .expect("commit remote");
        run_git(&remote_workdir, vec!["push".to_string()], None).expect("push remote");

        write_file(dir.path(), "README.md", "local draft\n");

        let result = pull_core(path, Some("origin".to_string()), true).expect("smart pull");
        assert!(!result.ok);
        assert!(result.message.contains("三栏合并窗口"));
        assert!(!result.output.contains("would be overwritten"));

        let state = operation_state_core(path).expect("operation state");
        assert_eq!(state.conflicted_paths, vec!["README.md".to_string()]);

        let details =
            conflict_details_core(path, "README.md".to_string()).expect("conflict details");
        let combined = format!(
            "{}{}",
            details.ours.as_deref().unwrap_or_default(),
            details.theirs.as_deref().unwrap_or_default()
        );
        assert!(combined.contains("remote change"));
        assert!(combined.contains("local draft"));
        assert_eq!(details.current_side, "theirs");
        assert_eq!(details.incoming_side, "ours");
        assert_eq!(details.conflict_source.as_deref(), Some("autostash"));
        assert!(details
            .ours
            .as_deref()
            .unwrap_or_default()
            .contains("remote change"));
        assert!(details
            .theirs
            .as_deref()
            .unwrap_or_default()
            .contains("local draft"));
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
    fn diff_for_untracked_file_includes_workdir_text() {
        let (dir, _repo) = test_repo();
        write_file(dir.path(), "notes.txt", "one\ntwo\n");

        let diff = get_diff_core(
            dir.path().to_str().unwrap(),
            Some("notes.txt".to_string()),
            false,
        )
        .expect("diff");

        assert!(diff.text.contains("+one"));
        assert_eq!(diff.old_text, None);
        assert_eq!(diff.new_text.as_deref(), Some("one\ntwo\n"));
        assert!(!diff.hunks.is_empty());
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
    fn commit_details_preserve_chinese_file_paths() {
        let (dir, _repo) = test_repo();
        initial_commit(dir.path());
        let file_path = "pages/chuan2/携程童/index.js";
        write_file(dir.path(), file_path, "console.log('hello');\n");
        stage_paths_core(dir.path().to_str().unwrap(), vec![file_path.to_string()]).expect("stage");
        let commit = commit_core(dir.path().to_str().unwrap(), "add chinese path".to_string())
            .expect("commit");

        let details =
            commit_details_core(dir.path().to_str().unwrap(), commit.oid).expect("details");
        assert_eq!(details.files[0].path, file_path);
        assert_eq!(details.files[0].status, "A");
    }

    #[test]
    fn name_status_parser_handles_nul_delimited_renames() {
        let files = parse_name_status("R100\0旧目录/旧文件.txt\0新目录/新文件.txt\0");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, "R100");
        assert_eq!(files[0].old_path.as_deref(), Some("旧目录/旧文件.txt"));
        assert_eq!(files[0].path, "新目录/新文件.txt");
    }

    #[test]
    fn porcelain_status_parser_handles_common_states() {
        let (files, counts) = parse_porcelain_status(
            " M src/main.rs\0A  staged.txt\0?? vendor/autoload.php\0!! cache/tmp.txt\0UU conflict.txt\0R  new.txt\0old.txt\0",
        );

        assert_eq!(files.len(), 6);
        assert_eq!(counts.staged, 2);
        assert_eq!(counts.unstaged, 2);
        assert_eq!(counts.untracked, 1);
        assert_eq!(counts.ignored, 1);
        assert_eq!(counts.conflicted, 1);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[0].kind, "modified");
        assert!(files[0].unstaged);
        assert_eq!(files[2].path, "vendor/autoload.php");
        assert!(files[2].untracked);
        assert_eq!(files[4].kind, "conflicted");
        assert_eq!(files[5].path, "new.txt");
        assert_eq!(files[5].old_path.as_deref(), Some("old.txt"));
        assert_eq!(files[5].kind, "renamed");
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
    fn conflict_analysis_classifies_common_block_shapes() {
        let same =
            analyze_conflict_block(&conflict_block(0, Some("base\n"), "changed\n", "changed\n"))
                .expect("same change");
        assert_eq!(same.kind, "same_change");
        assert_eq!(same.confidence, "certain");
        assert_eq!(same.suggested_side.as_deref(), Some("ours"));
        assert_eq!(same.replacement.as_deref(), Some("changed\n"));

        let one_side =
            analyze_conflict_block(&conflict_block(1, Some("base\n"), "base\n", "incoming\n"))
                .expect("one side");
        assert_eq!(one_side.kind, "one_side_change");
        assert_eq!(one_side.suggested_side.as_deref(), Some("theirs"));
        assert_eq!(one_side.replacement.as_deref(), Some("incoming\n"));

        let delete = analyze_conflict_block(&conflict_block(2, Some("remove\n"), "", "remove\n"))
            .expect("delete");
        assert_eq!(delete.kind, "delete_no_change");
        assert_eq!(delete.suggested_side.as_deref(), Some("ours"));
        assert_eq!(delete.replacement.as_deref(), Some(""));

        let whitespace =
            analyze_conflict_block(&conflict_block(3, None, "alpha beta\n", "alpha   beta\n"))
                .expect("whitespace");
        assert_eq!(whitespace.kind, "whitespace_only");
        assert_eq!(whitespace.confidence, "high");

        let non_overlapping = analyze_conflict_block(&conflict_block(
            4,
            Some("one\ntwo\nmiddle\nthree\n"),
            "one\nTWO\nmiddle\nthree\n",
            "one\ntwo\nmiddle\nTHREE\n",
        ))
        .expect("non-overlapping");
        assert_eq!(non_overlapping.kind, "non_overlapping");
        let replacement = non_overlapping.replacement.expect("replacement");
        assert!(replacement.contains("TWO"));
        assert!(replacement.contains("THREE"));

        let complex =
            analyze_conflict_block(&conflict_block(5, Some("base\n"), "ours\n", "theirs\n"))
                .expect("complex");
        assert_eq!(complex.kind, "complex");
        assert_eq!(complex.confidence, "low");
        assert!(complex.replacement.is_none());
    }

    #[test]
    fn preview_merge_is_read_only_and_reports_manual_conflict() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        initial_commit(dir.path());

        create_branch_core(path, "feature/preview-conflict".to_string(), true, None)
            .expect("create feature");
        write_file(dir.path(), "README.md", "feature\n");
        stage_paths_core(path, vec!["README.md".to_string()]).expect("stage feature");
        commit_core(path, "feature change".to_string()).expect("commit feature");

        checkout_branch_core(path, "master".to_string()).expect("checkout master");
        write_file(dir.path(), "README.md", "master\n");
        stage_paths_core(path, vec!["README.md".to_string()]).expect("stage master");
        commit_core(path, "master change".to_string()).expect("commit master");

        let before = run_git(
            dir.path(),
            vec!["status".to_string(), "--porcelain".to_string()],
            None,
        )
        .expect("status before");
        let preview = preview_merge_core(path, "feature/preview-conflict".to_string())
            .expect("preview merge");
        let after = run_git(
            dir.path(),
            vec!["status".to_string(), "--porcelain".to_string()],
            None,
        )
        .expect("status after");

        assert_eq!(before, after);
        assert!(!preview.clean);
        assert_eq!(preview.summary.manual, 1);
        assert_eq!(preview.summary.clean, 0);
        let readme = preview
            .files
            .iter()
            .find(|file| file.path == "README.md")
            .expect("README result");
        assert_eq!(readme.category, "manual");
        assert!(readme.conflict_count > 0);
        assert!(!readme.auto_resolvable);
    }

    #[test]
    fn preview_merge_reports_binary_and_add_delete_without_worktree_changes() {
        let (dir, _repo) = test_repo();
        let path = dir.path().to_str().unwrap();
        initial_commit(dir.path());

        fs::write(dir.path().join("binary.bin"), [0_u8, 1, 2, 3]).expect("write binary base");
        write_file(dir.path(), "remove-me.txt", "base\n");
        stage_paths_core(
            path,
            vec!["binary.bin".to_string(), "remove-me.txt".to_string()],
        )
        .expect("stage base files");
        commit_core(path, "base preview files".to_string()).expect("commit base files");

        create_branch_core(path, "feature/preview-nontext".to_string(), true, None)
            .expect("create feature");
        fs::write(dir.path().join("binary.bin"), [0_u8, 1, 9, 3]).expect("write binary feature");
        write_file(dir.path(), "remove-me.txt", "feature update\n");
        stage_paths_core(
            path,
            vec!["binary.bin".to_string(), "remove-me.txt".to_string()],
        )
        .expect("stage feature files");
        commit_core(path, "feature preview files".to_string()).expect("commit feature files");

        checkout_branch_core(path, "master".to_string()).expect("checkout master");
        fs::write(dir.path().join("binary.bin"), [0_u8, 4, 2, 3]).expect("write binary master");
        fs::remove_file(dir.path().join("remove-me.txt")).expect("remove file");
        stage_paths_core(
            path,
            vec!["binary.bin".to_string(), "remove-me.txt".to_string()],
        )
        .expect("stage master files");
        commit_core(path, "master preview files".to_string()).expect("commit master files");

        let before = run_git(
            dir.path(),
            vec!["status".to_string(), "--porcelain".to_string()],
            None,
        )
        .expect("status before");
        let preview =
            preview_merge_core(path, "feature/preview-nontext".to_string()).expect("preview merge");
        let after = run_git(
            dir.path(),
            vec!["status".to_string(), "--porcelain".to_string()],
            None,
        )
        .expect("status after");

        assert_eq!(before, after);
        assert!(!preview.clean);
        assert_eq!(preview.summary.binary, 1);
        assert_eq!(preview.summary.add_delete, 1);

        let binary = preview
            .files
            .iter()
            .find(|file| file.path == "binary.bin")
            .expect("binary result");
        assert_eq!(binary.category, "binary");
        assert!(!binary.auto_resolvable);

        let removed = preview
            .files
            .iter()
            .find(|file| file.path == "remove-me.txt")
            .expect("add/delete result");
        assert_eq!(removed.category, "add_delete");
        assert_eq!(removed.conflict_count, 1);
        assert!(!removed.auto_resolvable);
    }

    #[test]
    fn conflict_details_derives_blocks_after_markers_are_removed() {
        let (dir, _repo) = test_repo();
        write_file(
            dir.path(),
            "README.md",
            "Multiline merge conflict demo\n\nRelease checklist:\n- update the parser\n- refresh the sidebar\n- run merge viewer tests\n- write release notes\n\nOwner:\nname = base owner\nreview = pending\n",
        );
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage base");
        commit_core(dir.path().to_str().unwrap(), "base".to_string()).expect("commit base");

        create_branch_core(
            dir.path().to_str().unwrap(),
            "feature/multiline-conflict".to_string(),
            true,
            None,
        )
        .expect("create feature");
        write_file(
            dir.path(),
            "README.md",
            "Multiline merge conflict demo\n\nRelease checklist:\n- replace the parser flow with incoming\n- rebuild the sidebar badges\n- run merge viewer tests again\n- publish release notes from incoming\n\nOwner:\nname = incoming branch owner\nreview = blocked until remote check\n",
        );
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage feature");
        commit_core(dir.path().to_str().unwrap(), "incoming change".to_string())
            .expect("commit feature");

        checkout_branch_core(dir.path().to_str().unwrap(), "master".to_string())
            .expect("checkout master");
        write_file(
            dir.path(),
            "README.md",
            "Multiline merge conflict demo\n\nRelease checklist:\n- update the parser with current branch notes\n- refresh the sidebar after tests\n- run merge viewer tests in current branch\n- write release notes from local QA\n\nOwner:\nname = current branch owner\nreview = ready after local QA\n",
        );
        stage_paths_core(dir.path().to_str().unwrap(), vec!["README.md".to_string()])
            .expect("stage master");
        commit_core(dir.path().to_str().unwrap(), "current change".to_string())
            .expect("commit master");

        merge_branch_core(
            dir.path().to_str().unwrap(),
            "feature/multiline-conflict".to_string(),
            false,
            false,
            false,
        )
        .expect("merge conflict");
        write_file(
            dir.path(),
            "README.md",
            "Multiline merge conflict demo\n\nRelease checklist:\n- update the parser\n- refresh the sidebar\n- run merge viewer tests\n- write release notes\n\nOwner:\nname = base owner\nreview = pending\n",
        );

        let details = conflict_details_core(dir.path().to_str().unwrap(), "README.md".to_string())
            .expect("conflict details");
        assert_eq!(details.blocks.len(), 2);
        assert!(details.blocks[0].ours.contains("current branch notes"));
        assert!(details.blocks[0].theirs.contains("incoming"));
        assert!(details.blocks[1].ours.contains("current branch owner"));
        assert!(details.blocks[1].theirs.contains("incoming branch owner"));
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
            "manual result\n==========\nkept separator\n".to_string(),
            true,
        )
        .expect("save result");
        operation_control_core(dir.path().to_str().unwrap(), "continue".to_string())
            .expect("merge continue");

        let state = operation_state_core(dir.path().to_str().unwrap()).expect("state");
        assert!(!state.active);
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "manual result\n==========\nkept separator\n"
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
        let initialized_branches =
            list_branches_core(&initialized.path).expect("initialized branches");
        let initialized_main = initialized_branches
            .branches
            .iter()
            .find(|branch| branch.name == "main")
            .expect("initialized main branch");
        assert_eq!(initialized_branches.current.as_deref(), Some("main"));
        assert_eq!(initialized_main.branch_type, "local");
        assert!(initialized_main.current);
        assert!(initialized_main.target.is_none());
        assert_eq!(
            initialized.path,
            path_string(&fs::canonicalize(&init_path).unwrap())
        );
        assert_eq!(
            initialized.workdir.as_deref(),
            Some(initialized.path.as_str())
        );

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
