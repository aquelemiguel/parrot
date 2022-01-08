use parrot::client::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().unwrap();

    let mut parrot = Client::new(None).await?;
    if let Err(why) = parrot.start().await {
        println!("Fatality! Parrot crashed because: {:?}", why);
    };

    Ok(())
}
