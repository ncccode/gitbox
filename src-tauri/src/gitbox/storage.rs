pub fn record_recent_repo(app: &AppHandle, info: &RepositoryInfo) -> Result<(), GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    let name = Path::new(&info.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("仓库")
        .to_string();
    conn.execute(
        "INSERT INTO recent_repositories(path, name, branch, last_opened_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET
           name = excluded.name,
           branch = excluded.branch,
           last_opened_at = excluded.last_opened_at",
        params![info.path, name, info.branch, now_unix()],
    )?;
    Ok(())
}

pub fn record_shelf(
    app: &AppHandle,
    repo_path: &str,
    draft: ShelfDraft,
) -> Result<ShelfInfo, GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    conn.execute(
        "INSERT INTO shelves(repo_path, message, stash_ref, stash_oid, created_at, applied_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
        params![
            repo_path,
            draft.message,
            draft.stash_ref,
            draft.stash_oid,
            draft.created_at
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(ShelfInfo {
        id: Some(id),
        repo_path: repo_path.to_string(),
        message: draft.message,
        stash_ref: draft.stash_ref,
        stash_oid: draft.stash_oid,
        created_at: draft.created_at,
        applied_at: None,
    })
}

pub fn mark_shelf_applied(app: &AppHandle, stash_ref: &str) -> Result<(), GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    conn.execute(
        "UPDATE shelves SET applied_at = ?1 WHERE stash_ref = ?2 AND applied_at IS NULL",
        params![now_unix(), stash_ref],
    )?;
    Ok(())
}

pub fn list_shelves_core(app: &AppHandle, repo_path: &str) -> Result<Vec<ShelfInfo>, GitboxError> {
    let conn = storage_conn(app)?;
    init_storage(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, message, stash_ref, stash_oid, created_at, applied_at
         FROM shelves
         WHERE repo_path = ?1
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map(params![repo_path], |row| {
        Ok(ShelfInfo {
            id: row.get(0)?,
            repo_path: row.get(1)?,
            message: row.get(2)?,
            stash_ref: row.get(3)?,
            stash_oid: row.get(4)?,
            created_at: row.get(5)?,
            applied_at: row.get(6)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
