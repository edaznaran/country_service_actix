use std::{env, sync::LazyLock};

use futures::StreamExt;
use lapin::{BasicProperties, Connection, ConnectionProperties, options::*, types::FieldTable};
use serde_json::Value;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static QUEUE: LazyLock<String> =
    LazyLock::new(|| env::var("RABBITMQ_QUEUE").expect("RABBITMQ_QUEUE env var not set"));

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    // Inicializar el logger
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // connect to RabbitMQ server
    let host = env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST env var not set");
    let port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".into());
    let user = env::var("RABBITMQ_USER").expect("RABBITMQ_USER env var not set");
    let password = env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD env var not set");
    let vhost = env::var("RABBITMQ_VHOST").expect("RABBITMQ_VHOST env var not set");

    let conn = Connection::connect(
        &format!("amqp://{}:{}@{}:{}/{}", user, password, host, port, vhost), // rabbitMQ connection string
        ConnectionProperties::default(), // default connection properties
    )
    .await
    .expect("Failed to connect to RabbitMQ");

    let channel = conn
        .create_channel()
        .await
        .expect("Failed to create channel");
    let queue_name = QUEUE.as_str();

    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "rpc_server",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    info!("RPC Consumer is waiting for messages.");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("Error in delivery");

        // Extraer las propiedades para saber a dónde responder
        let reply_to = delivery.properties.reply_to().clone().unwrap().to_string();
        let correlation_id = delivery
            .properties
            .correlation_id()
            .clone()
            .unwrap()
            .to_string();
        let payload_str = std::str::from_utf8(&delivery.data).unwrap();
        let response = format!(
            "Received RPC request with correlation_id {}: {}",
            correlation_id, payload_str,
        );
        info!("{}", response);

        // --- Lógica de negocio ---
        // Aquí procesas el payload y generas una respuesta.
        if let Ok(json_payload) = serde_json::from_str::<Value>(payload_str) {
            match json_payload
                .get("pattern")
                .and_then(|p| p.get("cmd"))
                .and_then(|c| c.as_str())
            {
                Some("findByCriteria") => {
                    info!("Processing findByCriteria command");
                    // Aquí iría la lógica específica para este comando
                }
                Some("create") => {
                    info!("Processing create command");
                    // Aquí iría la lógica específica para este comando
                }
                Some("update") => {
                    info!("Processing update command");
                    // Aquí iría la lógica específica para este comando
                }
                _ => {
                    info!("Unknown command");
                }
            }
        } else {
            info!("Failed to parse payload as JSON");
        }

        let response_payload = serde_json::json!({"data": response}).to_string();
        // let response_payload = format!("Response to '{}'", payload_str);
        // -------------------------

        // Publicar la respuesta a la cola `reply_to`
        channel
            .basic_publish(
                "",        // default exchange
                &reply_to, // routing_key es el nombre de la cola de respuesta
                BasicPublishOptions::default(),
                response_payload.as_bytes(),
                BasicProperties::default().with_correlation_id(correlation_id.into()),
            )
            .await
            .expect("Failed to publish response");

        // Confirmar el mensaje original
        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("Failed to ack");
    }
}
