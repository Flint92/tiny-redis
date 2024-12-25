use bytes::{Bytes, BytesMut};
use tokio::io::Result;
use crate::resp::RespError;

const CR: u8 = b'\r';
const LF: u8 = b'\n';

/// This enum is wrapper for the different data types in RESP
#[derive(Debug, Clone)]
pub enum RespType {
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings>
    SimpleString(String),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-strings>
    BulkString(String),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-errors>
    SimpleError(String),
}

impl RespType {

    /// Parse the buffer and return the RESP type and the number of bytes consumed
    pub fn parse(buf: BytesMut) -> Result<(RespType, usize)> {
        let c = buf[0] as char;
        match c {
            '$' => Self::parse_bulk_string(buf),
            '+' => Self::parse_simple_string(buf),
            _ => Err(RespError::Other(String::from("Invalid RESP type")).into()),
        }
    }

    /// Convert the RESP value into its byte values
    pub fn to_bytes(&self) -> Bytes {
        match self {
            RespType::SimpleString(s) => Bytes::from(format!("+{}\r\n", s)),
            RespType::BulkString(s) => Bytes::from(format!("${}\r\n{}\r\n", s.chars().count(), s)),
            RespType::SimpleError(s) => Bytes::from(format!("-{}\r\n", s)),
        }
    }

    /// Parse the buffer into a SimpleString RESP value, and the number of bytes consumed
    fn parse_simple_string(buf: BytesMut) -> Result<(RespType, usize)> {
        if let Some((buf_data, len)) = Self::read_util_crlf(&buf[1..buf.len()]) {
            let simple_str = String::from_utf8(buf_data.to_vec());
            match simple_str {
                Ok(s) => Ok((RespType::SimpleString(s), len + 1)),
                Err(_) => Err(RespError::InvalidSimpleString(String::from(
                    "Simple string value is not a valid UTF-8 string",
                )).into()),
            }
        } else {
            Err(RespError::InvalidSimpleString(String::from("Invalid Simple String")).into())
        }
    }

    /// Parse the buffer into a BulkString RESP value, and the number of bytes consumed
    pub fn parse_bulk_string(buf: BytesMut) -> Result<(RespType, usize)> {
        let (blk_str_len, bytes_consumed) =
            if let Some((buf_data, len)) = Self::read_util_crlf(&buf[1..buf.len()]){
                let blk_str_len = Self::parse_usize_from_buf(buf_data)?;
                (blk_str_len, len + 1)
            } else {
                return Err(RespError::InvalidBulkString(String::from("Invalid Bulk String")).into());
            };

        // validate if buffer contains the complete string data based on
        // the length parsed in the previous step.
        let bulkstr_end_idx = bytes_consumed + blk_str_len as usize;
        if bulkstr_end_idx >= buf.len() {
            return Err(RespError::InvalidBulkString(String::from(
                "Invalid value for bulk string length",
            )).into())
        }

        // convert the bytes to a UTF-8 string
        let bulkstr = String::from_utf8(buf[bytes_consumed..bulkstr_end_idx].to_vec());
        match bulkstr {
            Ok(s) => Ok((RespType::BulkString(s), bulkstr_end_idx + 2)),
            Err(_) => Err(RespError::InvalidBulkString(String::from(
                "Bulk string value is not a valid UTF-8 string",
            )).into()),
        }
    }

    // Read the bytes till reaching CRLF("\r\n")
    fn read_util_crlf(buf: &[u8]) -> Option<(&[u8], usize)> {
        for i in 1..buf.len() {
            if buf[i - 1] == CR && buf[i] == LF {
                return Some((&buf[0..i - 1], i + 1));
            }
        }

        None
    }

    // Parse usize from bytes. The number is provided as a string in the buffer.
    // So convert raw bytes into UTF-8 string and then parse the string into usize.
    fn parse_usize_from_buf(p0: &[u8]) -> Result<usize> {
        let utf8_str = String::from_utf8(p0.to_vec());
        let parsed_int = match utf8_str {
            Ok(s) => {
                let i = s.parse::<usize>();
                match i {
                    Ok(n) => n,
                    Err(_) => {
                        return Err(RespError::Other(String::from("Invalid integer")).into());
                    }
                }
            },
            Err(_) => {
                return Err(RespError::Other(String::from("Invalid UTF-8 string")).into());
            },
        };

        Ok(parsed_int)
    }


}