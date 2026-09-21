use std::env;

use async_pop::response::types::DataType;

use rs_archiver::{
    email::parse_email,
    storage::{AttachmentKey, AttachmentStatus, Store},
    upload::{UploadReq, call_upload},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let pop3_domain = env::var("POP_DOMAIN").expect("POP_DOMAIN not set");
    let user_name = env::var("POP_USER").expect("POP_USER not set");
    let user_pass = env::var("POP_PASS").expect("POP_PASS not set");
    let uploader_user_name = env::var("CDA_USER").expect("CDA_USER not set");
    let uploader_user_pass = env::var("CDA_PASS").expect("CDA_PASS not set");
    let app_domain = env::var("CDA_DOMAIN").expect("CDA_DOMAIN not set");
    let cda_endpoint = env::var("CDA_ENDPOINT").expect("CDA_ENDPOINT not set");

    let store = Store::load_or_create("state.json").await?;

    let tls = async_native_tls::TlsConnector::new();
    let mut client =
        async_pop::connect((pop3_domain.as_str(), 995), pop3_domain.as_str(), &tls).await?;
    println!("connected to client");
    client.login(user_name, user_pass).await?;
    println!("user loged in");

    let messages = client.list(None).await?;
    let mut uidls: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    if let async_pop::response::uidl::UidlResponse::Multiple(list) = client.uidl(None).await? {
        for item in list.items() {
            if let Ok(id) = item.id().value() {
                let index = item.index().value()?;
                uidls.insert(index, id.to_string());
            }
        }
    }
    match messages {
        async_pop::response::list::ListResponse::Multiple(list) => {
            for stat in list.items() {
                let number = match stat.counter().value() {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("skipp : stat message number fails: {e}");
                        continue;
                    }
                };

                let uidl = match uidls.get(&number) {
                    Some(uidl) => uidl,
                    None => {
                        eprintln!("skip msg : no uidl found for message {number}");
                        continue;
                    }
                };

                let data = match client.retr(number).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("skip: retreve data failed for message {number}: {e}");
                        continue;
                    }
                };

                let email = match parse_email(&data) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("skip: parse failed for message {number}: {e}");
                        continue;
                    }
                };

                let attachment_names: Vec<String> = email
                    .attachments
                    .iter()
                    .map(|a| a.filename.clone())
                    .collect();

                for attachment in &email.attachments {
                    let key = AttachmentKey {
                        uidl: uidl.clone(),
                        attachment: attachment.filename.clone(),
                    };

                    if !store.is_uploadable(&key) {
                        println!("skipping already-resolved: {}", attachment.filename);
                        continue;
                    }

                    let status: AttachmentStatus = call_upload(
                        &cda_endpoint,
                        UploadReq {
                            username: uploader_user_name.clone(),
                            password: uploader_user_pass.clone(),
                            email: email.email.clone(),
                            domain: app_domain.clone(),
                            title: email.title.clone(),
                            filename: attachment.filename.clone(),
                            contents: attachment.contents.clone(),
                        },
                    )
                    .await;

                    match store.record_outcome(key, status).await {
                        Err(e) => {
                            println!(
                                "record_outcome failed att-name: {} msg-uidl:{} : {}",
                                attachment.filename,
                                uidl,
                                e.to_string()
                            )
                        }
                        _ => {}
                    }
                }
                if attachment_names.is_empty() {
                    println!("skipp msg with no attch : uidl={}", uidl);
                } else if store.message_is_fully_resolved(uidl, &attachment_names) {
                    if let Err(e) = client.dele(number).await {
                        eprintln!("failed to del message {number} (uidl {uidl}): {e}");
                    } else {
                        println!("deleted message {number} (uidl {uidl})");
                    }
                } else {
                    println!(
                        "leaving message {number} (uidl {uidl}, {}) -- still has retryable work",
                        email.email
                    );
                }
            }
        }
        _ => {}
    }

    client.quit().await?;
    Ok(())
}
