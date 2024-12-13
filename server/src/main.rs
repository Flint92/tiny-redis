mod resp;

use crate::resp::{RespHandler, Value};
use anyhow::Result;
use bytes::BufMut;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(stream: TcpStream) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".into()),
                "echo" => args.first().unwrap().clone(),
                "byte" => break,
                c => panic!("unknown command {}", c),
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
   match value {
       Value::Array(values) => {
           Ok((
               values.first().unwrap().serialize(),
               values.into_iter().skip(1).collect(),
           ))
       },
       Value::SimpleString(s) => Ok((s, vec![])),
       Value::Integer(i) => Ok((String::new(), vec![])), // todo
       Value::Error(e) => Ok((e, vec![])),
       Value::BulkString(s) => Ok((s, vec![])),
       _ => Err(anyhow::anyhow!("Unexpected command format")),
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