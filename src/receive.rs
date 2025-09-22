use futures::StreamExt;
use rabbitmq_stream_client::error::StreamCreateError;
use rabbitmq_stream_client::types::{ByteCapacity, OffsetSpecification, ResponseCode};
use std::env;
use std::io::stdin;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Failed to load .env file");
    use rabbitmq_stream_client::Environment;
    // connect to RabbitMQ server
    let host = env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST env var not set");
    let port: u16 = env::var("RABBITMQ_PORT")
        .unwrap_or_else(|_| "5672".into())
        .parse()
        .expect("RABBITMQ_PORT must be a valid port number");
    let user = env::var("RABBITMQ_USER").expect("RABBITMQ_USER env var not set");
    let password = env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD env var not set");
    let vhost = env::var("RABBITMQ_VHOST").expect("RABBITMQ_VHOST env var not set");

    let environment = Environment::builder()
        .host(&host)
        .port(port)
        .username(&user)
        .password(&password)
        .virtual_host(&vhost)
        .build()
        .await?;
    let stream = "hello-rust-stream";
    let create_response = environment
        .stream_creator()
        .max_length(ByteCapacity::GB(5))
        .create(stream)
        .await;

    if let Err(e) = create_response {
        if let StreamCreateError::Create { stream, status } = e {
            match status {
                // we can ignore this error because the stream already exists
                ResponseCode::StreamAlreadyExists => {}
                err => {
                    println!("Error creating stream: {:?} {:?}", stream, err);
                }
            }
        }
    }

    let mut consumer = environment
        .consumer()
        .offset(OffsetSpecification::First)
        .build(stream)
        .await
        .unwrap();

    let handle = consumer.handle();
    task::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            let d = delivery.unwrap();
            println!(
                "Got message: {:#?} with offset: {}",
                d.message()
                    .data()
                    .map(|data| String::from_utf8(data.to_vec()).unwrap()),
                d.offset(),
            );
        }
    });

    println!("Press any key to close the consumer");
    _ = stdin().read_line(&mut "".to_string());

    handle.close().await?;
    println!("consumer closed successfully");
    Ok(())
}
