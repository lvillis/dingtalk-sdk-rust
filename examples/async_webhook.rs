use dingtalk_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = match std::env::var("DINGTALK_WEBHOOK_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Set DINGTALK_WEBHOOK_TOKEN to run this example.");
            return Ok(());
        }
    };
    let secret = std::env::var("DINGTALK_WEBHOOK_SECRET").ok();

    let client = Client::builder().build()?;
    let webhook = client.webhook(token, secret);

    let response = webhook
        .send_text_message(
            "hello from dingtalk-sdk async webhook example",
            None,
            None,
            Some(false),
        )
        .await?;

    println!("{response}");
    Ok(())
}
