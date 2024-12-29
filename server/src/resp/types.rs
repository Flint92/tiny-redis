use crate::resp::RespError;
use bytes::{Bytes, BytesMut};

const CR: u8 = b'\r';
const LF: u8 = b'\n';

/// This enum is wrapper for the different data types in RESP
#[derive(Debug, Clone)]
pub enum RespType {
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings>
    SimpleString(String),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-strings>
    BulkString(String),
    /// Null representation in RESP2. It's simply a BulkString with length of negative one (-1).
    NullBulkString,
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-errors>
    SimpleError(String),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#arrays>
    Array(Vec<RespType>),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#integers>
    Integer(i64),
}

#[allow(dead_code)]
impl RespType {
    /// Parse the buffer and return the RESP type and the number of bytes consumed
    pub fn parse(buf: BytesMut) -> Result<(RespType, usize), RespError> {
        let c = buf[0] as char;
        match c {
            '$' => Self::parse_bulk_string(buf),
            '+' => Self::parse_simple_string(buf),
            '*' => Self::parse_array(buf),
            _ => Err(RespError::Other(String::from("Invalid RESP type"))),
        }
    }

    /// Convert the RESP value into its byte values
    pub fn to_bytes(&self) -> Bytes {
        match self {
            RespType::SimpleString(s) => Bytes::from(format!("+{}\r\n", s)),
            RespType::BulkString(s) => Bytes::from(format!("${}\r\n{}\r\n", s.chars().count(), s)),
            RespType::NullBulkString => Bytes::from("$-1\r\n"),
            RespType::SimpleError(s) => Bytes::from(format!("-{}\r\n", s)),
            RespType::Integer(i) => Bytes::from_iter(format!(":{}\r\n", i).into_bytes()),
            RespType::Array(arr) => {
                let mut arr_bytes = format!("*{}\r\n", arr.len()).into_bytes();
                arr.iter()
                    .map(|item| item.to_bytes())
                    .for_each(|b| arr_bytes.extend(b));
                Bytes::from_iter(arr_bytes)
            }
        }
    }

    /// Parse the buffer into an Array RESP value, and the number of bytes consumed
    pub fn parse_array(buf: BytesMut) -> Result<(RespType, usize), RespError> {
        let (arr_len, bytes_consumed) =
            if let Some((buf_data, len)) = Self::read_util_crlf(&buf[1..buf.len()]) {
                let arr_len = Self::parse_usize_from_buf(buf_data)?;
                (arr_len, len + 1)
            } else {
                return Err(RespError::InvalidArray(String::from("Invalid Array")));
            };

        // Parse the array elements
        let mut bytes_consumed = bytes_consumed;
        let mut arr = Vec::new();
        for _ in 0..arr_len {
            if let Ok((resp, bytes_consumed_next)) =
                Self::parse(BytesMut::from(&buf[bytes_consumed..buf.len()]))
            {
                if let RespType::SimpleError(_) = resp {
                    return Err(RespError::InvalidArray(String::from("Invalid Array")));
                }
                arr.push(resp);
                bytes_consumed += bytes_consumed_next
            } else {
                return Err(RespError::InvalidArray(String::from("Invalid Array")));
            }
        }

        Ok((RespType::Array(arr), bytes_consumed))
    }

    /// Parses the length of a RESP array from the given byte buffer.
    pub fn parse_array_len(buf: BytesMut) -> Result<(usize, usize), RespError> {
        if let Some((buf_data, len)) = Self::read_util_crlf(&buf[..]) {
            if len < 4 || buf_data[0] as char != '*' {
                return Err(RespError::InvalidArray(String::from("Not a valid RESP array")));
            }

            let arr_len = Self::parse_usize_from_buf(&buf_data[1..])?;
            Ok((arr_len, len))
        } else {
            Err(RespError::InvalidArray(String::from("Invalid Array")))
        }
    }

    /// Parse the buffer into a SimpleString RESP value, and the number of bytes consumed
    pub fn parse_simple_string(buf: BytesMut) -> Result<(RespType, usize), RespError> {
        if let Some((buf_data, len)) = Self::read_util_crlf(&buf[1..buf.len()]) {
            let simple_str = String::from_utf8(buf_data.to_vec());
            match simple_str {
                Ok(s) => Ok((RespType::SimpleString(s), len + 1)),
                Err(_) => Err(RespError::InvalidSimpleString(String::from(
                    "Simple string value is not a valid UTF-8 string",
                ))),
            }
        } else {
            Err(RespError::InvalidSimpleString(String::from("Invalid Simple String")))
        }
    }

    /// Parse the buffer into a BulkString RESP value, and the number of bytes consumed
    pub fn parse_bulk_string(buf: BytesMut) -> Result<(RespType, usize), RespError> {
        let (blkstr_len, bytes_consumed) = if let Some((buf_data, len)) =
            Self::read_util_crlf(&buf[1..buf.len()])
        {
            let blk_str_len = Self::parse_usize_from_buf(buf_data)?;
            (blk_str_len, len + 1)
        } else {
            return Err(RespError::InvalidBulkString(String::from("Invalid Bulk String")));
        };

        // validate if buffer contains the complete string data based on
        // the length parsed in the previous step.
        let bulkstr_end_idx = bytes_consumed + blkstr_len;
        if bulkstr_end_idx >= buf.len() {
            return Err(RespError::InvalidBulkString(String::from(
                "Invalid value for bulk string length",
            )));
        }

        // convert the bytes to a UTF-8 string
        let bulkstr = String::from_utf8(buf[bytes_consumed..bulkstr_end_idx].to_vec());
        match bulkstr {
            Ok(s) => Ok((RespType::BulkString(s), bulkstr_end_idx + 2)),
            Err(_) => Err(RespError::InvalidBulkString(String::from(
                "Bulk string value is not a valid UTF-8 string",
            ))),
        }
    }

    /// Parses the length of a RESP bulk string from the given byte buffer.
    pub fn parse_bulk_string_len(buf: BytesMut) -> Result<(usize, usize), RespError> {
        if let Some((buf_data, len)) = Self::read_util_crlf(&buf[..]) {
            if len < 4 || buf_data[0] as char != '$' {
                return Err(RespError::InvalidBulkString(String::from(
                    "Not a valid RESP bulk string",
                )));
            }

            let blkstr_len = Self::parse_usize_from_buf(&buf_data[1..])?;
            Ok((blkstr_len, len))
        } else {
            Err(RespError::InvalidBulkString(String::from("Invalid Bulk String")))
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
    fn parse_usize_from_buf(p0: &[u8]) -> Result<usize, RespError> {
        let utf8_str = String::from_utf8(p0.to_vec());
        let parsed_int = match utf8_str {
            Ok(s) => {
                let i = s.parse::<usize>();
                match i {
                    Ok(n) => n,
                    Err(_) => {
                        return Err(RespError::Other(String::from("Invalid integer")));
                    }
                }
            }
            Err(_) => {
                return Err(RespError::Other(String::from("Invalid UTF-8 string")));
            }
        };

        Ok(parsed_int)
    }
}
