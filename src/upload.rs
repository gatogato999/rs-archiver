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
pub async fn call_upload(url: &str, req: UploadReq) -> AttachmentStatus {
    let res = reqwest::Client::new().post(url).json(&req).send().await;

    match res {
        Ok(value) => {
            let data: Result<UploadRes, reqwest::Error> = value.json().await;
            match data {
                Ok(r) => {
                    if r.message == "Empty attachment" || r.message == "Ignoring small attachments"
                    {
                        return AttachmentStatus::SkippedTooSmall;
                    } else if r.success {
                        return AttachmentStatus::Uploaded;
                    }
                    return AttachmentStatus::FailedRetryable;
                }
                Err(e) => {
                    eprintln!("deserilazation failed : {e}");
                    return AttachmentStatus::FailedRetryable;
                }
            }
        }
        Err(e) => {
            eprintln!("call to uploader failed {e}");
            return AttachmentStatus::FailedRetryable;
        }
    };
}
