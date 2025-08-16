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

    let mut latency_list: Vec<f64> = Vec::new();

    let mut latency_count = 0;
    while let Some(message) = reader.next().await {
            
            let message_received_time = Utc::now();

            let json: serde_json::Value = serde_json::from_str(message.unwrap().to_text().unwrap()).unwrap();

            if json["table"] == "quote" {
                let message_timestamp = DateTime::parse_from_rfc3339(json["data"][0]["timestamp"].as_str().unwrap()).unwrap();
                let latency_in_ms = message_received_time.signed_duration_since(message_timestamp).num_microseconds().unwrap() as f64 / 1000.0;

                    // ignore outliers
                    if latency_in_ms < 10.0 {
                        latency_count += 1;

                        latency_list.push(latency_in_ms);
                        if latency_list.len() > 1000 {
                            latency_list.remove(0); // Remove the oldest value
                        }
                        println!("{} average: {:.3} ms", latency_in_ms, latency_list.iter().copied().sum::<f64>() / latency_count as f64);
                    } else {
                        println!("{} latency too high", latency_in_ms);
                    }
            }
        }
        
}
