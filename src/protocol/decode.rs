use std::collections::HashMap;

use super::RObject;
use anyhow::*;

const CRLF: &str = "\r\n";

impl RObject {
    
    pub fn decode(data: &str, start: usize) -> Result<(RObject, usize)> {
        if start >= data.len() {
            bail!("No data to decode");
        }
        match &data[start..=start] {
            "+" => Self::decode_simple_string(data, start),
            "-" => Self::decode_simple_error(data, start),
            ":" => Self::decode_integer(data, start),
            "$" => Self::decode_bulk_string(data, start),
            "*" => Self::decode_array(data, start),
            "_" => Ok((RObject::Null, start + 1 + CRLF.len())),
            "#" => Self::decode_boolean(data, start),
            "," => Self::decode_double(data, start),
            "(" => Self::decode_big_number(data, start),
            "!" => Self::decode_bulk_error(data, start),
            "=" => Self::decode_verbatim_string(data, start),
            "%" => Self::decode_map(data, start),
            "~" => Self::decode_set(data, start),
            ">" => Self::decode_push(data, start),
            _ => bail!("Unknown type"),
        }
    }

    fn decode_simple_string(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "+" {
            bail!("No + found at start of simple string");
        }
        cur += 1;
        // return the part between + and \r\n
        let end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in simple string"))? + cur;
        let simple_string = RObject::SimpleString(data[cur..end].to_string());
        cur = end + CRLF.len();

        Ok((simple_string, cur))
    }

    fn decode_simple_error(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "-" {
            bail!("No - found at start of simple error");
        }
        cur += 1;
        // return the part between - and \r\n
        let end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in simple error"))? + cur;
        let simple_error = RObject::SimpleError(data[cur..end].to_string());
        cur = end + CRLF.len();

        Ok((simple_error, cur))
    }

    fn decode_integer(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != ":" {
            bail!("No : found at start of integer");
        }
        cur += 1;
        // return the part between : and \r\n
        let end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in integer"))? + cur;
        let integer = RObject::Integer(data[cur..end].parse()?);
        cur = end + CRLF.len();

        Ok((integer, cur))
    }

    fn decode_bulk_string(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "$" {
            bail!("No $ found at start of bulk string");
        }
        cur += 1;
        // $<length><CRLF><content><CRLF>
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in bulk string length"))? + cur;
        let maybe_length = data[cur..length_end].parse::<i128>()?;
        if maybe_length == -1 {
            cur += CRLF.len();
            return Ok((RObject::NullBulkString, cur));
        }
        let length = maybe_length as usize;
        cur = length_end + CRLF.len();
        let content_end = cur + length;
        let bulk_string = RObject::BulkString(data[cur..content_end].to_string());
        cur = content_end + CRLF.len();
        Ok((bulk_string, cur))
    }

    fn decode_array(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "*" {
            bail!("No * found at start of array");
        }
        cur += 1;
        // *<length><CRLF><content>
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in array length"))? + cur;
        let maybe_length = data[cur..length_end].parse::<i128>()?;
        if maybe_length == -1 {
            cur += CRLF.len();
            return Ok((RObject::NullArray, cur));
        }
        let length = maybe_length as usize;
        cur = length_end + CRLF.len();
        let mut array = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let (element, new_cur) = RObject::decode(data, cur)?;
            array.push(element);
            cur = new_cur;
        }
        Ok((RObject::Array(array), cur))
    }

    fn decode_boolean(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "#" {
            bail!("No # found at start of boolean");
        }
        cur += 1;
        // #<t|f>CRLF
        let boolean = RObject::Boolean(data[cur..=cur] == *"t");
        cur += CRLF.len();
        Ok((boolean, cur))
    }

    fn decode_double(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "," {
            bail!("No , found at start of double");
        }
        cur += 1;

        let end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in double"))? + cur;
        let double_str = &data[cur..end];
        let double_val = double_str.parse::<f64>().map_err(|_| anyhow!("Failed to parse double: {}", double_str))?;
        let double = RObject::Double(double_val);
        cur = end + CRLF.len();

        Ok((double, cur))
    }

    fn decode_big_number(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "(" {
            bail!("No ( found at start of big number");
        }
        cur += 1;

        let end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in big number"))? + cur;
        let big_number = RObject::BigNumber(data[cur..end].to_string());
        cur = end + CRLF.len();

        Ok((big_number, cur))
    }

    fn decode_bulk_error(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "!" {
            bail!("No ! found at start of bulk error");
        }
        cur += 1;
        // !<length>\r\n<error>\r\n
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in bulk error length"))? + cur;
        let length = data[cur..length_end].parse::<usize>()?;
        cur = length_end + CRLF.len();
        let error_end = cur + length;
        let bulk_error = RObject::BulkError(data[cur..error_end].to_string());
        cur = error_end + CRLF.len();
        Ok((bulk_error, cur))
    }

    fn decode_verbatim_string(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "=" {
            bail!("No = found at start of verbatim string");
        }
        cur += 1;
        // =<length>\r\n<encoding>\r\n<content>\r\n
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in verbatim string length"))? + cur;
        let length = data[cur..length_end].parse::<usize>()?;
        cur = length_end + CRLF.len();
        let encoding_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in verbatim string encoding"))? + cur;
        let encoding = data[cur..encoding_end].to_string();
        cur = encoding_end + CRLF.len();
        let content_end = cur + length;
        let verbatim_string = RObject::VerbatimString(data[cur..content_end].to_string(), encoding);
        cur = content_end + CRLF.len();
        Ok((verbatim_string, cur))
    }

    fn decode_map(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "%" {
            bail!("No % found at start of map");
        }
        cur += 1;
        // %<length>\r\n<key><value>...
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in map length"))? + cur;
        let maybe_length = data[cur..length_end].parse::<i128>()?;
        if maybe_length == -1 {
            cur += CRLF.len();
            return Ok((RObject::Null, cur));
        }
        let length = maybe_length as usize;
        cur = length_end + CRLF.len();
        let mut map = HashMap::with_capacity(length as usize);
        for _ in 0..length {
            let (key, new_cur) = RObject::decode(data, cur)?;
            let (value, new_cur) = RObject::decode(data, new_cur)?;
            map.insert(key, value);
            cur = new_cur;
        }
        Ok((RObject::Map(map), cur))
    }

    fn decode_set(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != "~" {
            bail!("No ~ found at start of set");
        }
        cur += 1;
        // ~<length>\r\n<element>...
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in set length"))? + cur;
        let maybe_length = data[cur..length_end].parse::<i128>()?;
        if maybe_length == -1 {
            cur += CRLF.len();
            return Ok((RObject::Null, cur));
        }
        let length = maybe_length as usize;
        cur = length_end + CRLF.len();
        let mut set = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let (element, new_cur) = RObject::decode(data, cur)?;
            set.push(element);
            cur = new_cur;
        }
        Ok((RObject::Set(set), cur))
    }

    fn decode_push(data: &str, start: usize) -> Result<(RObject, usize)> {
        let mut cur = start;
        if &data[cur..=cur] != ">" {
            bail!("No > found at start of push");
        }
        cur += 1;
        // ><length>\r\n<element>...
        let length_end = data[cur..].find(CRLF).ok_or_else(|| anyhow!("No CRLF found in push length"))? + cur;
        let maybe_length = data[cur..length_end].parse::<i128>()?;
        if maybe_length == -1 {
            cur += CRLF.len();
            return Ok((RObject::Null, cur));
        }
        let length = maybe_length as usize;
        cur = length_end + CRLF.len();
        let mut push = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let (element, new_cur) = RObject::decode(data, cur)?;
            push.push(element);
            cur = new_cur;
        }
        Ok((RObject::Push(push), cur))
    }
}