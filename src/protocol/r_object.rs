use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fmt::Display;
use std::fmt;
#[derive(Debug)]
pub enum RObject {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    NullBulkString,
    Array(Vec<RObject>),
    NullArray,
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(String),
    BulkError(String),
    VerbatimString(String, String),
    Map(HashMap<RObject, RObject>),
    Set(Vec<RObject>),
    Push(Vec<RObject>),
}

impl Display for RObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RObject::SimpleString(s) => write!(f, "+{}\r\n", s),
            RObject::SimpleError(s) => write!(f, "-{}\r\n", s),
            RObject::Integer(i) => write!(f, ":{}\r\n", i),
            RObject::BulkString(s) => write!(f, "${}\r\n{}\r\n", s.len(), s),
            RObject::NullBulkString => write!(f, "$-1\r\n"),
            RObject::Array(a) => {
                write!(f, "*{}\r\n", a.len())?;
                for item in a {
                    write!(f, "{}", item)?;
                }
                Ok(())
            },
            RObject::NullArray => write!(f, "*-1\r\n"),
            RObject::Null => write!(f, "_\r\n"),
            RObject::Boolean(b) => write!(f, "#{}\r\n", if *b { "t" } else { "f" }),
            RObject::Double(d) => write!(f, "${}\r\n{}", d.to_string().len(), d),
            RObject::BigNumber(s) => write!(f, ":{}\r\n", s),
            RObject::BulkError(s) => write!(f, "!{}\r\n{}\r\n", s.len(), s),
            RObject::VerbatimString(s, e) => write!(f, "={}\r\n{}:{}\r\n", s.len(), e, s),
            RObject::Map(m) => {
                write!(f, "%{}\r\n", m.len())?;
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort_by_key(|&k| k.to_string());
                for key in keys {
                    write!(f, "{}{}", key, m.get(key).unwrap())?;
                }
                Ok(())
            },
            RObject::Set(s) => {
                write!(f, "*{}\r\n", s.len())?;
                for item in s {
                    write!(f, "{}", item)?;
                }
                Ok(())
            },
            RObject::Push(s) => {
                write!(f, ">{}\r\n", s.len())?;
                for item in s {
                    write!(f, "{}", item)?;
                }
                Ok(())
            },
        }
    }
}

impl Hash for RObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RObject::SimpleString(s) => s.hash(state),
            RObject::SimpleError(s) => s.hash(state),
            RObject::Integer(i) => i.hash(state),
            RObject::BulkString(s) => s.hash(state),
            RObject::NullBulkString => state.write(RObject::NullBulkString.to_string().as_bytes()),
            RObject::Array(a) => {
                state.write_u8(b'*');
                for item in a {
                    item.hash(state);
                }
            },
            RObject::NullArray => state.write(RObject::NullArray.to_string().as_bytes()),
            RObject::Null => state.write_u8(0),
            RObject::Boolean(b) => b.hash(state),
            RObject::Double(d) => d.to_bits().hash(state),
            RObject::BulkError(s) => s.hash(state),
            RObject::BigNumber(s) => s.hash(state),
            RObject::VerbatimString(s, e) => {
                s.hash(state);
                e.hash(state);
            },
            RObject::Map(m) => {
                state.write_u8(b'%');
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort_by_key(|&k| k.to_string());
                for key in keys {
                    key.hash(state);
                    m.get(key).unwrap().hash(state);
                }
            },
            RObject::Set(s) => {
                state.write_u8(b'*');
                for item in s {
                    item.hash(state);
                }
            },
            RObject::Push(s) => {
                state.write_u8(b'>');
                for item in s {
                    item.hash(state);
                }
            },
        }
    }
}

impl PartialEq for RObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RObject::SimpleString(a), RObject::SimpleString(b)) => a == b,
            (RObject::SimpleError(a), RObject::SimpleError(b)) => a == b,
            (RObject::Integer(a), RObject::Integer(b)) => a == b,
            (RObject::BulkString(a), RObject::BulkString(b)) => a == b,
            (RObject::Array(a), RObject::Array(b)) => a == b,
            (RObject::Null, RObject::Null) => true,
            (RObject::Boolean(a), RObject::Boolean(b)) => a == b,
            (RObject::Double(a), RObject::Double(b)) => a == b,
            (RObject::BigNumber(a), RObject::BigNumber(b)) => a == b,
            (RObject::VerbatimString(a, e), RObject::VerbatimString(b, f)) => a == b && e == f,
            (RObject::Map(a), RObject::Map(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().all(|(key, value)| {
                    b.get(key).map_or(false, |v| value == v)
                })
            },
            (RObject::Set(a), RObject::Set(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for RObject {}
