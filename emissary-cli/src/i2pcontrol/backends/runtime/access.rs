//! Bounded access policy for accepted-stream server tunnels.
//!
//! Access entries are resolved once, while the tunnel configuration is being
//! validated.  The accepted-stream path only compares the trusted SAM-derived
//! Destination hash; it never parses request headers, performs DNS, or does
//! filesystem/network I/O.

use std::{
    collections::HashSet,
    fs,
    path::{Component, Path},
};

use emissary_core::crypto::base32_decode;

use super::{admission::PeerKey, TrustedPeerIdentity};

pub const MAX_ACCESS_ENTRIES: usize = 1024;
pub const MAX_ACCESS_ENTRY_LENGTH: usize = 1024;
pub const MAX_FILTER_FILE_BYTES: u64 = 64 * 1024;
pub const MAX_FILTER_FILE_ENTRIES: usize = 1024;
pub const MAX_FILTER_FILE_ENTRY_LENGTH: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessOption {
    Allow,
    Deny,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerAccessPolicy {
    option: AccessOption,
    entries: Option<HashSet<PeerKey>>,
}

impl Default for ServerAccessPolicy {
    fn default() -> Self {
        Self {
            option: AccessOption::Allow,
            entries: None,
        }
    }
}

impl ServerAccessPolicy {
    pub fn from_values(option: Option<&str>, list: Option<&str>) -> Result<Self, &'static str> {
        let option = match option.unwrap_or("allow") {
            "allow" => AccessOption::Allow,
            "deny" => AccessOption::Deny,
            _ => return Err("AccessOption"),
        };
        let Some(list) = list else {
            return Ok(Self {
                option,
                entries: None,
            });
        };
        if list.len() > MAX_ACCESS_ENTRIES * MAX_ACCESS_ENTRY_LENGTH {
            return Err("AccessList");
        }
        let mut entries = HashSet::new();
        for entry in list.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
            if entry.len() > MAX_ACCESS_ENTRY_LENGTH || entries.len() >= MAX_ACCESS_ENTRIES {
                return Err("AccessList");
            }
            entries.insert(parse_entry(entry).ok_or("AccessList")?);
        }
        Ok(Self {
            option,
            entries: (!entries.is_empty()).then_some(entries),
        })
    }

    /// Load a newline-delimited access/filter generation beneath `root`.
    ///
    /// The file is fully validated and parsed before the returned generation
    /// can be published by a backend.  Callers retain their previous policy
    /// if this function fails, which gives reloads transactional semantics.
    pub fn from_filter_file(root: &Path, relative_path: &str) -> Result<Self, &'static str> {
        if relative_path.is_empty()
            || relative_path.len() > MAX_FILTER_FILE_ENTRY_LENGTH
            || Path::new(relative_path).is_absolute()
            || Path::new(relative_path)
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err("FilterFilePath");
        }
        let candidate = root.join(relative_path);
        let root = root.canonicalize().map_err(|_| "FilterFilePath")?;
        let canonical = candidate.canonicalize().map_err(|_| "FilterFilePath")?;
        if !canonical.starts_with(&root) {
            return Err("FilterFilePath");
        }
        let metadata = fs::metadata(&canonical).map_err(|_| "FilterFilePath")?;
        if !metadata.is_file() || metadata.len() > MAX_FILTER_FILE_BYTES {
            return Err("FilterFilePath");
        }
        let contents = fs::read_to_string(&canonical).map_err(|_| "FilterFilePath")?;
        let mut entries = HashSet::new();
        for line in contents.lines() {
            let entry = line.trim();
            if entry.is_empty() || entry.starts_with('#') {
                continue;
            }
            if entry.len() > MAX_FILTER_FILE_ENTRY_LENGTH
                || entries.len() >= MAX_FILTER_FILE_ENTRIES
            {
                return Err("FilterFilePath");
            }
            entries.insert(parse_entry(entry).ok_or("FilterFilePath")?);
        }
        if entries.is_empty() {
            return Err("FilterFilePath");
        }
        Ok(Self {
            option: AccessOption::Allow,
            entries: Some(entries),
        })
    }

    pub fn allows(&self, peer: &TrustedPeerIdentity) -> bool {
        let Some(entries) = self.entries.as_ref() else {
            return true;
        };
        let matches = entries.contains(&PeerKey::from_identity(peer));
        match self.option {
            AccessOption::Allow => matches,
            AccessOption::Deny => !matches,
        }
    }

    pub fn option(&self) -> AccessOption {
        self.option
    }
}

fn parse_entry(entry: &str) -> Option<PeerKey> {
    if let Some(identity) = TrustedPeerIdentity::from_destination_text(entry) {
        return Some(PeerKey::from_identity(&identity));
    }
    let entry = entry.strip_suffix(".b32.i2p").unwrap_or(entry);
    if entry.len() != 52
        || !entry
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'2'..=b'7'))
    {
        return None;
    }
    let decoded = base32_decode(entry)?;
    if decoded.len() != 32 {
        return None;
    }
    let mut key = [0; 32];
    key.copy_from_slice(&decoded);
    Some(PeerKey::from_bytes(key))
}

#[cfg(test)]
mod tests {
    use super::{super::peer_identity::test_fixtures::distinct_peer, *};
    use emissary_core::crypto::base32_encode;

    #[test]
    fn canonicalizes_full_and_base32_peer_entries() {
        let peer = distinct_peer(1);
        let b32 = base32_encode(peer.canonical_id());
        let policy = ServerAccessPolicy::from_values(None, Some(&b32)).unwrap();
        assert!(policy.allows(&peer));
        assert!(!ServerAccessPolicy::from_values(Some("deny"), Some(&b32)).unwrap().allows(&peer));
    }

    #[test]
    fn empty_access_list_does_not_change_default_admission() {
        let peer = distinct_peer(3);
        let policy = ServerAccessPolicy::from_values(Some("deny"), None).unwrap();
        assert!(policy.allows(&peer));
    }

    #[test]
    fn rejects_malformed_and_oversized_access_lists() {
        assert!(ServerAccessPolicy::from_values(None, Some("not-a-destination")).is_err());
        assert!(ServerAccessPolicy::from_values(
            None,
            Some(&"x".repeat(MAX_ACCESS_ENTRIES * MAX_ACCESS_ENTRY_LENGTH + 1))
        )
        .is_err());
    }

    #[test]
    fn confined_filter_file_rejects_traversal_and_parses_a_generation() {
        let directory = tempfile::tempdir().unwrap();
        let peer = distinct_peer(2);
        let path = directory.path().join("access.txt");
        std::fs::write(&path, format!("{}\n# comment\n", peer.destination())).unwrap();
        let policy = ServerAccessPolicy::from_filter_file(directory.path(), "access.txt").unwrap();
        assert!(policy.allows(&peer));
        assert!(ServerAccessPolicy::from_filter_file(directory.path(), "../access.txt").is_err());
    }
}
