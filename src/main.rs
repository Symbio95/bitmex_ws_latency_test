use chrono::{DateTime, Utc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json;

/// Code to test latency on bitmex.com's websocket for deploying on different server setups
#[tokio::main]
async fn main() {

    // connect to the websocket server
    let (ws_stream, _) = connect_async("wss://www.bitmex.com/realtime").await.unwrap();

    // split the websocket stream into a writer and a reader
    let (mut writer, mut reader) = ws_stream.split();

    // subscribe to the XBTUSD quote
    let subscribe_msg = r#"{"op": "subscribe", "args": ["quote:XBTUSD"]}"#;
    let msg = Message::Text(subscribe_msg.into());
    writer.send(msg).await.unwrap();


    while let Some(message) = reader.next().await {
            
            let message_received_time = Utc::now();

            let json: serde_json::Value = serde_json::from_str(message.unwrap().to_text().unwrap()).unwrap();

            if let Some(quote_timestamp) = json["data"][0]["timestamp"].as_str() {

                let message_timestamp = DateTime::parse_from_rfc3339(quote_timestamp).unwrap();
                let latency_in_ms = message_received_time.signed_duration_since(message_timestamp).num_milliseconds();

                println!("Latency: {} ms", latency_in_ms);
            }
        }
        
}
