const DEFAULT_MAX_ATTEMPTS: u32 = 10;
use std::fs;
use std::{collections::HashMap, path::PathBuf};

pub struct Store {
    pub attachments: HashMap<AttachmentKey, AttachmentState>,
    pub path: PathBuf,
    pub max_attch_attempts: u32,
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
pub struct Entry {
    pub key: AttachmentKey,
    pub state: AttachmentState,
}

impl Store {
    pub fn load_or_create(path: impl Into<PathBuf>) -> Result<Store, Box<dyn std::error::Error>> {
        let path = path.into();
        let max_attempts = std::env::var("MAX_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MAX_ATTEMPTS);

        let attachments = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            let entries: Vec<Entry> = serde_json::from_str(&raw)?;
            entries.into_iter().map(|e| (e.key, e.state)).collect()
        } else {
            HashMap::new()
        };
        Ok(Store {
            attachments,
            path,
            max_attch_attempts: max_attempts,
        })
    }

    pub fn is_uploadable(&self, key: &AttachmentKey) -> bool {
        match self.attachments.get(key) {
            None => true,
            Some(state) => match state.status {
                AttachmentStatus::FailedRetryable => state.attempts < self.max_attch_attempts,
                _ => false,
            },
        }
    }

    pub fn record_outcome(
        &mut self,
        key: AttachmentKey,
        mut status: AttachmentStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let attempts = self.attachments.get(&key).map(|s| s.attempts).unwrap_or(0) + 1;
            if status == AttachmentStatus::FailedRetryable && attempts >= self.max_attch_attempts {
                status = AttachmentStatus::FailedPermanent;
            }
            self.attachments
                .insert(key, AttachmentState { status, attempts });
        }

        self.save()
    }

    pub fn message_is_fully_resolved(&self, uidl: &str, attachment_names: &[String]) -> bool {
        attachment_names.iter().all(|name| {
            let key = AttachmentKey {
                uidl: uidl.to_string(),
                attachment: name.clone(),
            };
            match self.attachments.get(&key).map(|s| s.status) {
                Some(AttachmentStatus::Uploaded) => true,
                Some(AttachmentStatus::SkippedTooSmall) => true,
                Some(AttachmentStatus::FailedPermanent) => true,
                _ => false,
            }
        })
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entries: Vec<Entry> = {
            self.attachments
                .iter()
                .map(|(k, v)| Entry {
                    key: k.clone(),
                    state: v.clone(),
                })
                .collect()
        };

        let json = serde_json::to_string_pretty(&entries)?;
        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
}
