use dingtalk_sdk::{Client, ContactGetUserRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let appkey = match std::env::var("DINGTALK_APP_KEY") {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "Set DINGTALK_APP_KEY, DINGTALK_APP_SECRET, DINGTALK_ROBOT_CODE and DINGTALK_USER_ID to run this example."
            );
            return Ok(());
        }
    };
    let appsecret = match std::env::var("DINGTALK_APP_SECRET") {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "Set DINGTALK_APP_KEY, DINGTALK_APP_SECRET, DINGTALK_ROBOT_CODE and DINGTALK_USER_ID to run this example."
            );
            return Ok(());
        }
    };
    let robot_code = match std::env::var("DINGTALK_ROBOT_CODE") {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "Set DINGTALK_APP_KEY, DINGTALK_APP_SECRET, DINGTALK_ROBOT_CODE and DINGTALK_USER_ID to run this example."
            );
            return Ok(());
        }
    };
    let user_id = match std::env::var("DINGTALK_USER_ID") {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "Set DINGTALK_APP_KEY, DINGTALK_APP_SECRET, DINGTALK_ROBOT_CODE and DINGTALK_USER_ID to run this example."
            );
            return Ok(());
        }
    };

    let client = Client::builder().build()?;
    let enterprise = client.enterprise(appkey, appsecret, robot_code);

    let user = enterprise
        .contact_get_user(ContactGetUserRequest::new(user_id))
        .await?;

    println!("{}", serde_json::to_string_pretty(&user)?);
    Ok(())
}
