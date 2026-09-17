#[derive(Debug)]
pub enum UploadErr {
    RequestErr,
    TimeoutErr,
    NetworkErr,
    IOErr,
}

impl From<reqwest::Error> for UploadErr {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            println!("timeout error: {e}");
            UploadErr::TimeoutErr
        } else if e.is_request() {
            println!("bad request error: {e}");
            UploadErr::RequestErr
        } else {
            println!("other connect error: {e}");
            UploadErr::NetworkErr
        }
    }
}
impl From<std::io::Error> for UploadErr {
    fn from(e: std::io::Error) -> Self {
        println!("io error: {e}");
        UploadErr::IOErr
    }
}
