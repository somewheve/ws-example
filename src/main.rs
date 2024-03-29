use futures::{SinkExt, StreamExt};
use xitca_client::bytes::Bytes;
use xitca_client::Client;
use xitca_client::ws::{Message, WebSocket};

const API: &str = "wss://stream.binance.com:443/ws";

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct App {
    pub socket: WebSocket<'static>,
}

impl App {
    pub async fn run() -> Result<(), Error> {
        let cli = Client::builder().rustls().finish();
        let sock = cli.ws(API)?.send().await?.leak();
        let message: Bytes = Bytes::from_static("ddd".as_ref());
        let mut app = App { socket: sock };
        app.socket.send(Message::Text(message)).await?;
        loop {
            match app.socket.next().await {
                Some(Ok(Message::Text(msg))) => {
                    println!("{:?}", msg);
                }
                Some(Ok(Message::Ping(msg))) => {
                    app.socket.send(Message::Pong(msg)).await?;
                }
                Some(Ok(Message::Close(close))) => {
                    if let Some(reason) = close {
                        println!("close reason: {:?}", reason);
                    }
                    break;
                }
                Some(Err(e)) => { return Err(Error::from(e)); }
                _ => {}
            }
        }

        Ok(())
    }
}

async fn run() {
    match App::run().await {
        Err(e) => {
            tracing::error!("app错误: {e}");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        Ok(_) => tracing::info!("服务器正常关闭连接"),
    }
}


fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap()
        .block_on(async { tokio::spawn(crate::run()).await })
        .unwrap()
}
