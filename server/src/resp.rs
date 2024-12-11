use anyhow::Error;
use bytes::BytesMut;
use tokio::net::TcpStream;
use anyhow::{anyhow, Result};

pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
}

impl Value {
    pub fn serialize(&self) -> String {
        match self {
            Value::SimpleString(s) => {
                format!("+{}\r\n", s)
            },
            Value::BulkString(s) => {
                format!("${}\r\n{}\r\n", s.chars().count(), s)
            },
            _ => "-Error unsupported command\r\n".to_string()
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler{
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Value> {
        // TODO read from socket and parse
        todo!()
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        // TODO serialise and write to the socket
        todo!()
    }

}

fn parse_message(buf: BytesMut) -> Result<(Value, usize)> {
    match buf[0] as char {
        '+' => parse_simple_string(buf),
        '*' => parse_array(buf),
        '$' => parse_bulk_string(buf),
        _ => Err(anyhow!("Not a known value type {}", buf)),
    }
}

fn parse_bulk_string(p0: BytesMut) -> Result<(Value, usize)> {
    todo!()
}

fn parse_array(p0: BytesMut) -> Result<(Value, usize)> {
    todo!()
}

fn parse_simple_string(buf: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        let str = String::from_utf8(line.to_vec())?;
        return Ok((Value::SimpleString(str), len + 1))
    }

    Err(anyhow!("Not a simple string {}", buf))
}

fn read_util_crlf(buf: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buf.len() {
        if buf[i-1] == b'\r' && buf[i] == b'\n' {
            return Some((&buf[0..(i - 1)], i + 1))
        }
    }

    None
}