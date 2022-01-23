use parrot::{client::Client, strings::FAIL_MAIN};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let mut parrot = Client::default().await?;
    if let Err(why) = parrot.start().await {
        println!("{}: {:?}", FAIL_MAIN, why);
    };

    Ok(())
}
