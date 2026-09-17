use crate::errors::UploadErr;
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
pub async fn call_upload(req: UploadReq) -> Result<UploadRes, UploadErr> {
    let res = reqwest::Client::new()
        .post("http://localhost:10032/cda/UploadAttachment")
        .json(&req)
        .send()
        .await?;
    let data: UploadRes = res.json().await?;
    Ok(data)
}
