use std::io::BufRead;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const CR: u8 = b'\r';
const LF: u8 = b'\n';

#[derive(Clone)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Error(String),
    Integer(i64),
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
            Value::Array(values) => {
                let mut s = format!("*{}\r\n", values.len());

                for value in values.iter() {
                    s += value.serialize().as_str();
                }

                s
            },
            Value::Error(s) => {
                format!("-{}\r\n", s)
            },
            Value::Integer(i) => {
                format!(":{}\r\n", i)
            },
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

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let mut  bytes = [0u8; 512];

        let bytes_read = self.stream.read(&mut bytes).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        self.buffer.extend_from_slice(&bytes[..bytes_read]);

        let (v, _) = parse_message(self.buffer.split())?;

        self.buffer.clear();

        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        self.stream.write(value.serialize().as_bytes()).await?;

        Ok(())
    }

}

fn parse_message(buf: BytesMut) -> Result<(Value, usize)> {
    match buf[0] as char {
        '+' => parse_simple_string(buf),
        '*' => parse_array(buf),
        '-' => parse_error(buf),
        ':' => parse_integer(buf),
        '$' => parse_bulk_string(buf),
        _ => Err(anyhow!("Not a known value type {:?}", buf)),
    }
}

fn parse_integer(buf: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        return Ok((Value::Integer(parse_int(line)?), len + 1))
    }

    Err(anyhow!("Not a error string {:?}", buf))
}

fn parse_error(buf: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        let str = String::from_utf8(line.to_vec())?;
        return Ok((Value::Error(str), len + 1))
    }

    Err(anyhow!("Not a error string {:?}", buf))
}

fn parse_bulk_string(buf: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow!("Invalid bulk array format {:?}", buf));
    };

    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;

    Ok((Value::BulkString(String::from_utf8(buf[bytes_consumed..end_of_bulk_str].to_vec())?), total_parsed))
}

fn parse_array(buf: BytesMut) -> Result<(Value, usize)> {
    let (array_len, mut bytes_consumed) = if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        let array_len = parse_int(line)?;
        (array_len, len + 1)
    } else {
        return Err(anyhow!("Invalid array format {:?}", buf));
    };

    let mut itmes = vec![];
    for _ in 0..array_len {
        let (array_item, len) = parse_message(BytesMut::from(&buf[bytes_consumed..]))?;
        itmes.push(array_item);
        bytes_consumed += len;
    }

    Ok((Value::Array(itmes), bytes_consumed))
}

fn parse_simple_string(buf: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_util_crlf(&buf[1..]) {
        let str = String::from_utf8(line.to_vec())?;
        return Ok((Value::SimpleString(str), len + 1))
    }

    Err(anyhow!("Not a simple string {:?}", buf))
}

fn parse_int(buf: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buf.to_vec())?.parse::<i64>()?)
}

fn read_util_crlf(buf: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buf.len() {
        if buf[i-1] == CR && buf[i] == LF {
            return Some((&buf[0..(i - 1)], i + 1))
        }
    }

    None
}