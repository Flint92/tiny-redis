mod resp;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::resp::{RespHandler, Value};

async fn handle_connection(stream: TcpStream) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = match value {
            // TODO handle echo and ping
            Value::SimpleString(_) => todo!(),
            Value::BulkString(_) => todo!(),
            Value::Array(_) => todo!(),
        };

        handler.write_value(response).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:16379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}