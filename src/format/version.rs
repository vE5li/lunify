use std::fmt::Display;

use crate::LunifyError;

/// List of supported Lua versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LuaVersion {
    /// Lua 5.0.*
    Lua50,
    /// Lua 5.1.*
    Lua51,
}

impl TryFrom<u8> for LuaVersion {
    type Error = LunifyError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x51 => Ok(LuaVersion::Lua51),
            0x50 => Ok(LuaVersion::Lua50),
            version => Err(LunifyError::UnsupportedVersion(version)),
        }
    }
}

impl From<LuaVersion> for u8 {
    fn from(value: LuaVersion) -> Self {
        match value {
            LuaVersion::Lua51 => 0x51,
            LuaVersion::Lua50 => 0x50,
        }
    }
}

impl Display for LuaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaVersion::Lua50 => write!(f, "Lua 5.0"),
            LuaVersion::Lua51 => write!(f, "Lua 5.1"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LuaVersion, LunifyError};

    #[test]
    fn lua51() {
        assert_eq!(0x51.try_into(), Ok(LuaVersion::Lua51));
    }

    #[test]
    fn lua50() {
        assert_eq!(0x50.try_into(), Ok(LuaVersion::Lua50));
    }

    #[test]
    fn unsupported_version() {
        assert_eq!(LuaVersion::try_from(0x52), Err(LunifyError::UnsupportedVersion(0x52)));
    }

    #[test]
    fn lua50_to_u8() {
        assert_eq!(u8::from(LuaVersion::Lua50), 0x50);
    }

    #[test]
    fn lua51_to_u8() {
        assert_eq!(u8::from(LuaVersion::Lua51), 0x51);
    }

    #[test]
    fn format_lua50() {
        assert_eq!(format!("{}", LuaVersion::Lua50).as_str(), "Lua 5.0");
    }

    #[test]
    fn format_lua51() {
        assert_eq!(format!("{}", LuaVersion::Lua51).as_str(), "Lua 5.1");
    }
}
