use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio::time::{Instant, interval};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to BitMEX websocket
    let url = "wss://www.bitmex.com/realtime";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Rolling buffer and pending map
    let mut rtts: VecDeque<f64> = VecDeque::with_capacity(1000);
    let mut pending: HashMap<u64, Instant> = HashMap::new();

    // Sequence id for ping payload
    let mut seq: u64 = 0;

    // Send a ping every second
    let mut ticker = interval(Duration::from_secs(1));

    println!("connected to {} - sending pings every 1s", url);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                seq = seq.wrapping_add(1);
                let payload = seq.to_be_bytes().to_vec();
                pending.insert(seq, Instant::now());
                // send ping with seq as payload (convert to Bytes)
                if let Err(e) = write.send(Message::Ping(payload.into())).await {
                    eprintln!("failed to send ping: {}", e);
                    break;
                }
            }

            msg = read.next() => {
                match msg {
                    Some(Ok(message)) => match message {
                        Message::Pong(payload) => {
                            if payload.len() >= 8 {
                                let mut b = [0u8;8];
                                b.copy_from_slice(&payload.as_ref()[0..8]);
                                let id = u64::from_be_bytes(b);
                                if let Some(sent) = pending.remove(&id) {
                                    let rtt_ms = sent.elapsed().as_secs_f64() * 1000.0;
                                    rtts.push_back(rtt_ms);
                                    if rtts.len() > 1000 { rtts.pop_front(); }

                                    // compute simple percentiles
                                    let mut v: Vec<f64> = rtts.iter().copied().collect();
                                    v.sort_by(|a,b| a.partial_cmp(b).unwrap());
                                    let p50 = percentile(&v, 50.0);
                                    let p95 = percentile(&v, 95.0);
                                    let p99 = percentile(&v, 99.0);

                                    println!("pong id={} rtt={:.3} ms  samples={} p50={:.3} p95={:.3} p99={:.3}", id, rtt_ms, v.len(), p50, p95, p99);
                                }
                            } else {
                                println!("received pong with empty payload");
                            }
                        }
                        Message::Close(frame) => {
                            println!("server closed connection: {:?}", frame);
                            break;
                        }
                        _ => {
                            // ignore other messages
                        }
                    },
                    Some(Err(e)) => {
                        eprintln!("websocket error: {}", e);
                        break;
                    }
                    None => {
                        println!("websocket stream ended");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn percentile(sorted: &Vec<f64>, pct: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let len = sorted.len();
    if len == 1 { return sorted[0]; }
    let rank = pct/100.0 * (len - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let w = rank - lower as f64;
        sorted[lower] * (1.0 - w) + sorted[upper] * w
    }
}
