use std::convert::Infallible;

/// What a handler is allowed to do with the packet it was registered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OwnType {
    ReadOnly,
    ReadWrite,
    ReadInject,
    ReadWriteInject,
}

impl TryFrom<OwnType> for &'static str {
    type Error = Infallible;
    fn try_from(value: OwnType) -> Result<Self, Self::Error> {
        Ok(match value {
            OwnType::ReadOnly => "read",
            OwnType::ReadWrite => "read-write",
            OwnType::ReadInject => "read-inject",
            OwnType::ReadWriteInject => "read-write-inject",
        })
    }
}

impl TryFrom<&str> for OwnType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "read" => OwnType::ReadOnly,
            "read-write" => OwnType::ReadWrite,
            "read-inject" => OwnType::ReadInject,
            "read-write-inject" => OwnType::ReadWriteInject,
            _ => return Err("Invalid OwnType"),
        })
    }
}
