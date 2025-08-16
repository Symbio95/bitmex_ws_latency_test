use chrono::Utc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

/// Code to test latency on bitmex.com's websocket for deploying on different server setups
#[tokio::main]
async fn main() {

    // connect to the websocket server
    let (ws_stream, _) = connect_async("wss://www.bitmex.com/realtime").await.unwrap();

    // split the websocket stream into a writer and a reader
    let (mut writer, mut reader) = ws_stream.split();

    let mut latency_sum: f64 = 0.0;
    let mut latency_count = 0;

    writer.send(Message::Ping("ping".into())).await.unwrap();

    let mut start = Utc::now();
    while let Some(_) = reader.next().await {
            let latency = Utc::now().signed_duration_since(start).num_microseconds().unwrap() as f64 / 1000.0;
            println!("latency: {:.3} ms", latency);
            if latency > 10.0 {
                continue;
            }
            latency_sum += latency;
            latency_count += 1;

            println!("latency: {:.3}", latency_sum / latency_count as f64);
            writer.send(Message::Ping("ping".into())).await.unwrap();
            start = Utc::now();
        }
        
}
