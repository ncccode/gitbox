use git2::{
    BranchType, DiffFormat, DiffOptions, ErrorCode, Oid, Repository, Status, StatusOptions,
};
use rusqlite::{params, Connection};
use serde::Serialize;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

include!("gitbox/types.rs");
include!("gitbox/git_errors.rs");
include!("gitbox/repo_project.rs");
include!("gitbox/changes_commit.rs");
include!("gitbox/remote.rs");
include!("gitbox/history_refs.rs");
include!("gitbox/operations_conflicts.rs");
include!("gitbox/storage.rs");
include!("gitbox/git_helpers.rs");

#[cfg(test)]
include!("gitbox/tests.rs");
