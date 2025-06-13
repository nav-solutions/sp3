//! sp3 version

use crate::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    /// SP3-a revision. See <https://igs.org/formats-and-standards/>
    A,

    /// SP3-b revision. See <https://igs.org/formats-and-standards/>
    B,

    /// SP3-c revision. See <https://igs.org/formats-and-standards/>
    C,

    #[default]
    /// SP3-d revision (latest). See <https://igs.org/formats-and-standards/>
    D,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::A => f.write_str("a"),
            Self::B => f.write_str("b"),
            Self::C => f.write_str("c"),
            Self::D => f.write_str("d"),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("a") {
            Ok(Self::A)
        } else if s.eq("b") {
            Ok(Self::B)
        } else if s.eq("c") {
            Ok(Self::C)
        } else if s.eq("d") {
            Ok(Self::D)
        } else {
            Err(ParsingError::NonSupportedRevision)
        }
    }
}

impl From<Version> for u8 {
    fn from(val: Version) -> Self {
        match val {
            Version::A => 1,
            Version::B => 2,
            Version::C => 3,
            Version::D => 4,
        }
    }
}

impl From<u8> for Version {
    fn from(lhs: u8) -> Version {
        match lhs {
            1 => Version::A,
            2 => Version::B,
            3 => Version::C,
            _ => Version::D,
        }
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        let s: u8 = self.into();
        (s + rhs).into()
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        let s: u8 = self.into();

        let mut s = s as i8;
        s -= rhs as i8;

        if s <= 0 {
            Self::A
        } else {
            (s as u8).into()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Version;
    use std::str::FromStr;

    #[test]
    fn version() {
        for (desc, expected) in [("c", Version::C), ("d", Version::D)] {
            let version = Version::from_str(desc);
            assert!(version.is_ok(), "failed to parse Version from \"{}\"", desc);
            assert_eq!(version.unwrap(), expected);
        }

        for (vers, expected) in [(Version::C, 3), (Version::D, 4)] {
            let version: u8 = vers.into();
            assert_eq!(version, expected, "convertion to integer failed");
        }

        assert!(Version::C < Version::D);
        assert!(Version::D >= Version::C);

        assert_eq!(Version::C - 2, Version::A);
        assert_eq!(Version::C - 1, Version::B);
        assert_eq!(Version::C + 1, Version::D);

        assert_eq!(Version::D - 3, Version::A);
        assert_eq!(Version::D - 2, Version::B);
        assert_eq!(Version::D - 1, Version::C);
        assert_eq!(Version::D + 1, Version::D);

        assert_eq!(Version::A - 1, Version::A);
        assert_eq!(Version::A + 1, Version::B);
        assert_eq!(Version::A + 2, Version::C);

        let version: Version = 4_u8.into();
        assert_eq!(version, Version::D);

        let version: Version = 3_u8.into();
        assert_eq!(version, Version::C);

        assert!(Version::A < Version::B);
        assert!(Version::A < Version::C);
        assert!(Version::A < Version::D);
        assert!(Version::D > Version::C);
    }
}
