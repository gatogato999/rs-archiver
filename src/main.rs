use std::env;

use pop3_mail_client::{Pop3Client, Pop3Connection};

use rs_archiver::process::process_messages;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let pop3_domain = env::var("POP_DOMAIN").expect("POP_DOMAIN not set");
    let user_name = env::var("POP_USER").expect("POP_USER not set");
    let user_pass = env::var("POP_PASS").expect("POP_PASS not set");
    let uploader_user_name = env::var("CDA_USER").expect("CDA_USER not set");
    let uploader_user_pass = env::var("CDA_PASS").expect("CDA_PASS not set");
    let app_domain = env::var("CDA_DOMAIN").expect("CDA_DOMAIN not set");
    let cda_endpoint = env::var("CDA_ENDPOINT").expect("CDA_ENDPOINT not set");

    let mut client = Pop3Client::builder()
        .username(&user_name)
        .password(&user_pass)
        .connect(Pop3Connection::new(pop3_domain.as_str(), 995))?;

    process_messages(
        &mut client,
        cda_endpoint.as_str(),
        uploader_user_name,
        uploader_user_pass,
        app_domain,
    )?;
    drop(client);
    Ok(())
}
