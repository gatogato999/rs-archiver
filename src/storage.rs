const DEFAULT_MAX_ATTEMPTS: u32 = 10;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::fs;

pub type SharedStore = Arc<Store>;

pub struct Store {
    attachments: Mutex<HashMap<AttachmentKey, AttachmentState>>,
    path: PathBuf,
    max_attch_attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttachmentStatus {
    Uploaded,
    SkippedTooSmall,
    FailedRetryable,
    FailedPermanent,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttachmentKey {
    pub uidl: String,
    pub attachment: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentState {
    pub status: AttachmentStatus,
    pub attempts: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Entry {
    key: AttachmentKey,
    state: AttachmentState,
}

impl Store {
    pub async fn load_or_create(
        path: impl Into<PathBuf>,
    ) -> Result<SharedStore, Box<dyn std::error::Error>> {
        let path = path.into();

        let max_attempts = std::env::var("MAX_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MAX_ATTEMPTS);

        let attachments = if path.exists() {
            let raw = fs::read_to_string(&path).await?;
            let entries: Vec<Entry> = serde_json::from_str(&raw)?;
            entries.into_iter().map(|e| (e.key, e.state)).collect()
        } else {
            HashMap::new()
        };
        Ok(Arc::new(Store {
            attachments: Mutex::new(attachments),
            path,
            max_attch_attempts: max_attempts,
        }))
    }

    pub fn is_uploadable(&self, key: &AttachmentKey) -> bool {
        let attachments = self.attachments.lock().unwrap();
        match attachments.get(key) {
            None => true,
            Some(state) => match state.status {
                AttachmentStatus::Uploaded => false,
                AttachmentStatus::SkippedTooSmall => false,
                AttachmentStatus::FailedPermanent => false,
                AttachmentStatus::FailedRetryable => state.attempts < self.max_attch_attempts,
            },
        }
    }

    pub async fn record_outcome(
        &self,
        key: AttachmentKey,
        mut status: AttachmentStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut attachments = self.attachments.lock().unwrap();
            let attempts = attachments.get(&key).map(|s| s.attempts).unwrap_or(0) + 1;
            if status == AttachmentStatus::FailedRetryable && attempts >= self.max_attch_attempts {
                status = AttachmentStatus::FailedPermanent;
            }
            attachments.insert(key, AttachmentState { status, attempts });
        }

        self.save().await
    }

    pub fn message_is_fully_resolved(&self, uidl: &str, attachment_names: &[String]) -> bool {
        let attachments = self.attachments.lock().unwrap();
        attachment_names.iter().all(|name| {
            let key = AttachmentKey {
                uidl: uidl.to_string(),
                attachment: name.clone(),
            };
            match attachments.get(&key).map(|s| s.status) {
                Some(AttachmentStatus::Uploaded) => true,
                Some(AttachmentStatus::SkippedTooSmall) => true,
                Some(AttachmentStatus::FailedPermanent) => true,
                _ => false,
            }
        })
    }

    async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entries: Vec<Entry> = {
            let attachments = self.attachments.lock().unwrap();
            attachments
                .iter()
                .map(|(k, v)| Entry {
                    key: k.clone(),
                    state: v.clone(),
                })
                .collect()
        };

        let json = serde_json::to_string_pretty(&entries)?;
        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, json).await?;
        fs::rename(&tmp_path, &self.path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test_helpers {
    use crate::storage::{AttachmentKey, AttachmentState, AttachmentStatus, Entry};

    pub fn create_entry(
        uidl: &str,
        attch_name: &str,
        status: AttachmentStatus,
        attemps: u8,
    ) -> Entry {
        Entry {
            state: AttachmentState {
                status: status,
                attempts: attemps as u32,
            },
            key: AttachmentKey {
                uidl: uidl.into(),
                attachment: attch_name.into(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::storage::AttachmentStatus::{
        FailedPermanent, FailedRetryable, SkippedTooSmall, Uploaded,
    };
    use crate::storage::Store;
    use crate::storage::test_helpers::create_entry;
    use crate::storage::{AttachmentKey, AttachmentState};
    use std::path::PathBuf;
    use std::{collections::HashMap, sync::Mutex};
    #[tokio::test]
    async fn test_create_or_upload() {
        struct TestCleanup {
            path: std::path::PathBuf,
        }

        impl Drop for TestCleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }
        let path = PathBuf::from("test.json");
        assert_eq!(!path.exists(), true);

        let temp = TestCleanup { path: path.clone() };
        if let Ok(s) = Store::load_or_create(temp.path.clone()).await {
            assert_eq!(s.attachments.lock().unwrap().is_empty(), true);
            assert_eq!(temp.path.exists(), false);
            let mut attchs = s.attachments.lock().unwrap();
            attchs.insert(
                AttachmentKey {
                    uidl: "A".into(),
                    attachment: "A".into(),
                },
                AttachmentState {
                    status: Uploaded,
                    attempts: 1,
                },
            );
            drop(attchs);
            if let Ok(()) = s.save().await {
                assert_eq!(s.attachments.lock().unwrap().is_empty(), false);
                assert_eq!(temp.path.exists(), true);
            }
        }
        drop(path);
    }
    #[test]
    fn test_is_uploadable() {
        let entry_vec = vec![
            create_entry("A", "A.pdf", Uploaded, 1),
            create_entry("B", "B.pdf", SkippedTooSmall, 1),
            create_entry("C", "C.pdf", FailedRetryable, 1),
            create_entry("D", "D.pdf", FailedRetryable, 9),
            create_entry("E", "E.pdf", FailedPermanent, 1),
        ];
        let h_map: HashMap<AttachmentKey, AttachmentState> =
            entry_vec.into_iter().map(|e| (e.key, e.state)).collect();

        let path = PathBuf::from("test.json");
        let store = Store {
            attachments: Mutex::new(h_map),
            path,
            max_attch_attempts: 2,
        };

        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "nwe".into(),
                attachment: "new-file.pdf".into()
            }),
            true
        );
        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "A".into(),
                attachment: "A.pdf".into()
            }),
            false
        );
        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "B".into(),
                attachment: "B.pdf".into()
            }),
            false
        );
        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "C".into(),
                attachment: "C.pdf".into()
            }),
            true
        );
        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "D".into(),
                attachment: "D.pdf".into()
            }),
            false
        );
        assert_eq!(
            store.is_uploadable(&AttachmentKey {
                uidl: "E".into(),
                attachment: "E.pdf".into()
            }),
            false
        );
    }
}
