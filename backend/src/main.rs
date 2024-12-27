use std::{collections::HashMap, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    let server = Server::default();

    while let Ok((tcp_stream, _)) = listener.accept().await {
        let server = server.clone();

        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(tcp_stream).await {
                let uuid = Uuid::now_v7();
                let (sender, mut receiver) = ws_stream.split();

                // 1. The client joins the server
                server.join(uuid, sender).await;

                // 2. The server broadcasts all messages received from the client to all other clients
                while let Some(Ok(message)) = receiver.next().await {
                    if message.is_text() {
                        server.broadcast(message).await;
                    }
                }

                // 3. The client leaves the server
                server.leave(uuid).await;
            }
        });
    }
}

#[derive(Clone, Default)]
struct Server {
    clients: Arc<Mutex<HashMap<Uuid, SplitSink<WebSocketStream<TcpStream>, Message>>>>,
}

impl Server {
    async fn join(&self, uuid: Uuid, sender: SplitSink<WebSocketStream<TcpStream>, Message>) {
        self.clients.lock().await.insert(uuid, sender);
    }

    async fn broadcast(&self, message: Message) {
        for (_, sender) in self.clients.lock().await.iter_mut() {
            let _ = sender.send(message.clone()).await;
        }
    }

    async fn leave(&self, uuid: Uuid) {
        self.clients.lock().await.remove(&uuid);
    }
}
