use crate::storage::AttachmentStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadReq {
    pub username: String,
    pub password: String,
    pub email: String,
    pub title: String,
    pub filename: String,
    pub contents: String,
    pub domain: String,
}
#[derive(Debug, Deserialize)]
pub struct UploadRes {
    success: bool,
    message: String,
}
pub struct Uploader {
    pub client: ureq::Agent,
}

impl Uploader {
    pub fn new() -> Self {
        Self {
            client: ureq::Agent::new_with_defaults(),
        }
    }
    pub fn call_upload(&self, url: &str, req: UploadReq) -> AttachmentStatus {
        let res = self.client.post(url).send_json(&req);

        match res {
            Ok(response) => match response.into_body().read_json::<UploadRes>() {
                Ok(r) => {
                    if r.message == "Empty attachment" || r.message == "Ignoring small attachments"
                    {
                        AttachmentStatus::SkippedTooSmall
                    } else if r.success {
                        AttachmentStatus::Uploaded
                    } else {
                        AttachmentStatus::FailedRetryable
                    }
                }
                Err(e) => {
                    eprintln!("deserialization failed: {e}");
                    AttachmentStatus::FailedRetryable
                }
            },
            Err(e) => {
                eprintln!("call to uploader failed: {e}");
                AttachmentStatus::FailedRetryable
            }
        }
    }
}
