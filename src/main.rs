use std::env;

use async_pop::response::types::DataType;

use crate::email::parse_email;

mod email;
mod errors;
mod upload;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let pop3_domain = env::var("POP_DOMAIN").expect("POP_DOMAIN (pop3_domain) not set");
    let user_name = env::var("POP_USER").expect("POP_USER (pop3_user) not set");
    let user_pass = env::var("POP_PASS").expect("POP_PASS (pop3_pssword) not set");

    let tls = async_native_tls::TlsConnector::new();
    let mut client =
        async_pop::connect((pop3_domain.as_str(), 995), pop3_domain.as_str(), &tls).await?;
    println!("connected to client");
    client.login(user_name, user_pass).await?;
    println!("user loged in");

    let messages = client.list(None).await?;
    match messages {
        async_pop::response::list::ListResponse::Multiple(list) => {
            for stat in list.items() {
                let number = stat.counter().value()?;
                let data = client.retr(number).await?;
                let email = parse_email(&data)?;
                for e in email.attachments {
                    println!("{}, {}", email.title, e.filename);
                    match client.uidl(Some(number)).await? {
                        async_pop::response::uidl::UidlResponse::Single(id) => {
                            if let Ok(msg_uidl) = id.id().value() {
                                println!("msg uid {:?}, msg num {}", msg_uidl, id.index());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
    client.quit().await?;
    Ok(())
}
