//! Small, I2PControl-owned filesystem publication primitives.
//!
//! The helpers keep serialization and store-specific validation in their
//! callers while sharing the bounded file, permission, and directory-sync
//! mechanics. Unsupported platforms intentionally report success for the
//! directory step because Rust does not expose an equivalent portable API.

use std::path::Path;

const MAX_PUBLICATION_NAME_LENGTH: usize = 128;

pub(crate) async fn ensure_directory(dir: &Path) -> Result<(), String> {
    reject_directory_link(dir).await?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| "failed to create publication directory".to_string())?;
    let metadata = tokio::fs::symlink_metadata(dir)
        .await
        .map_err(|_| "failed to inspect publication directory".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("publication path is not a regular directory".to_string());
    }
    Ok(())
}

pub(crate) async fn write_synced_file(
    path: &Path,
    bytes: &[u8],
    max_size: usize,
) -> Result<(), String> {
    if bytes.len() > max_size {
        return Err("publication payload exceeds its size limit".to_string());
    }
    reject_existing_file_link(path).await?;
    if tokio::fs::symlink_metadata(path).await.is_ok() {
        tokio::fs::remove_file(path)
            .await
            .map_err(|_| "failed to replace stale publication temporary file".to_string())?;
    }
    tokio::fs::write(path, bytes)
        .await
        .map_err(|_| "failed to write publication temporary file".to_string())?;
    set_restrictive_permissions(path).await?;
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_| "failed to open publication temporary file".to_string())?;
    file.sync_all().await.map_err(|_| "failed to sync publication file".to_string())
}

/// Publish a fixed current/backup pair. The caller updates live state only
/// after this function returns `Ok(())`.
pub(crate) async fn publish_with_backup(
    dir: &Path,
    current_name: &str,
    backup_name: &str,
    temp_name: &str,
    bytes: &[u8],
    max_size: usize,
) -> Result<(), String> {
    validate_name(current_name)?;
    validate_name(backup_name)?;
    validate_name(temp_name)?;
    ensure_directory(dir).await?;

    let current = dir.join(current_name);
    let backup = dir.join(backup_name);
    let temp = dir.join(temp_name);
    for path in [&current, &backup, &temp] {
        reject_existing_file_link(path).await?;
    }

    write_synced_file(&temp, bytes, max_size).await?;

    let mut rotated = false;
    if tokio::fs::symlink_metadata(&current).await.is_ok() {
        if tokio::fs::symlink_metadata(&backup).await.is_ok() {
            tokio::fs::remove_file(&backup)
                .await
                .map_err(|_| "failed to replace publication backup".to_string())?;
        }
        if let Err(error) = tokio::fs::rename(&current, &backup).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(format!("failed to preserve publication backup: {error}"));
        }
        rotated = true;
    }

    if let Err(error) = tokio::fs::rename(&temp, &current).await {
        let _ = tokio::fs::remove_file(&temp).await;
        if rotated {
            let _ = tokio::fs::rename(&backup, &current).await;
        }
        return Err(format!("failed to publish current state: {error}"));
    }

    // The prior generation remains in backup. The new current is complete, but
    // the caller must not publish it into live state on this failure.
    sync_directory(dir).await?;
    Ok(())
}

pub(crate) async fn sync_directory(dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let directory = tokio::fs::File::open(dir)
            .await
            .map_err(|_| "failed to open publication directory for sync".to_string())?;
        directory
            .sync_all()
            .await
            .map_err(|_| "failed to sync publication directory".to_string())?;
    }
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

pub(crate) fn sync_directory_sync(dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let directory = std::fs::File::open(dir)
            .map_err(|_| "failed to open publication directory for sync".to_string())?;
        directory
            .sync_all()
            .map_err(|_| "failed to sync publication directory".to_string())?;
    }
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

pub(crate) fn publish_with_backup_sync(
    dir: &Path,
    current_name: &str,
    backup_name: &str,
    temp_name: &str,
    bytes: &[u8],
    max_size: usize,
) -> Result<(), String> {
    validate_name(current_name)?;
    validate_name(backup_name)?;
    validate_name(temp_name)?;
    std::fs::create_dir_all(dir)
        .map_err(|_| "failed to create publication directory".to_string())?;
    let current = dir.join(current_name);
    let backup = dir.join(backup_name);
    let temp = dir.join(temp_name);
    for path in [&current, &backup, &temp] {
        reject_existing_file_link_sync(path)?;
    }
    if bytes.len() > max_size {
        return Err("publication payload exceeds its size limit".to_string());
    }
    write_synced_file_sync(&temp, bytes)?;
    if current.exists() {
        if backup.exists() {
            std::fs::remove_file(&backup)
                .map_err(|_| "failed to replace publication backup".to_string())?;
        }
        std::fs::rename(&current, &backup)
            .map_err(|_| "failed to preserve publication backup".to_string())?;
    }
    if let Err(error) = std::fs::rename(&temp, &current) {
        let _ = std::fs::remove_file(&temp);
        let _ = std::fs::rename(&backup, &current);
        return Err(format!("failed to publish current state: {error}"));
    }
    sync_directory_sync(dir)
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > MAX_PUBLICATION_NAME_LENGTH
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
    {
        return Err("invalid fixed publication filename".to_string());
    }
    Ok(())
}

async fn reject_directory_link(path: &Path) -> Result<(), String> {
    if tokio::fs::symlink_metadata(path)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("publication directory is a symlink".to_string());
    }
    Ok(())
}

async fn reject_existing_file_link(path: &Path) -> Result<(), String> {
    let Ok(metadata) = tokio::fs::symlink_metadata(path).await else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        return Err("publication file is a symlink".to_string());
    }
    if !metadata.is_file() {
        return Err("publication path is not a regular file".to_string());
    }
    Ok(())
}

fn reject_existing_file_link_sync(path: &Path) -> Result<(), String> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        return Err("publication file is a symlink".to_string());
    }
    if !metadata.is_file() {
        return Err("publication path is not a regular file".to_string());
    }
    Ok(())
}

async fn set_restrictive_permissions(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .await
            .map_err(|_| "failed to set publication file permissions".to_string())?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn write_synced_file_sync(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes)
        .map_err(|_| "failed to write publication temporary file".to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|_| "failed to set publication file permissions".to_string())?;
    }
    let file = std::fs::File::open(path)
        .map_err(|_| "failed to open publication temporary file".to_string())?;
    file.sync_all().map_err(|_| "failed to sync publication file".to_string())
}
