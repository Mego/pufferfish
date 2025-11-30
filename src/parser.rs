use std::collections::HashSet;

use grid::Grid;
use thiserror::Error;

use crate::program::Tank;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("duplicate name found: {0}")]
    DuplicateName(String),
    #[error("invalid name found: {0}")]
    InvalidName(String),
}

fn is_valid_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c == '\''
}

fn is_valid_name(name: &str) -> bool {
    !name.starts_with("'") && !name.contains("''") && !name.ends_with("'")
}

pub fn parse_names(code: &str) -> Result<HashSet<String>, ParseError> {
    let mut names = HashSet::new();
    let mut chars = code.chars().fuse();
    while let Some(c) = chars.next() {
        if !is_valid_name_char(c) {
            continue;
        }
        let mut name = format!("{c}");
        while let Some(c) = chars.next()
            && is_valid_name_char(c)
        {
            name.push(c);
        }
        if !is_valid_name(&name) {
            return Err(ParseError::InvalidName(name));
        }
        if !names.insert(name.clone()) {
            return Err(ParseError::DuplicateName(name));
        }
    }
    Ok(names)
}

impl Tank {
    fn from_mask_and_name(name: String, mask: &str) -> Result<Self, anyhow::Error> {
        let mut data = Vec::with_capacity(20);
        for x in mask.bytes() {
            let val = byte_to_hex(x);
            data.extend((0..4).map(move |i| (val >> i) & 1).rev());
        }
        Ok(Self::new(name, Grid::from_vec(data, 4)))
    }

    fn swizzle(mut self) -> Self {
        let mut elems = self.grid.into_vec();
        let (a, b) = elems.split_at_mut(11);
        a.swap_with_slice(b);
        self.grid = Grid::from_vec(elems, 4);
        self
    }
}

fn byte_to_hex(byte: u8) -> usize {
    (match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("invalid hex byte: {byte:x}"),
    }) as usize
}

const FONT: [&str; 26] = [
    "07997", "8e99e", "06886", "17997", "06bc6", "24e44", "79716", "88e99", "04044", "20224",
    "89ae9", "44442", "0edd9", "0e999", "06996", "e99e8", "79971", "0ac88", "07c3e", "4e442",
    "00997", "009a4", "09bb7", "0a44a", "99716", "0f24f",
];

pub fn populate_tanks(names: HashSet<String>) -> Result<Vec<Tank>, anyhow::Error> {
    names
        .into_iter()
        .map(|name| {
            name.bytes()
                .try_fold(Tank::new(name.clone(), Grid::new(5, 4)), |acc, x| {
                    if x == b'\'' {
                        Ok(acc.swizzle())
                    } else {
                        Tank::from_mask_and_name(Default::default(), FONT[(x - b'a') as usize])
                            .map(|t| acc + t)
                    }
                })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name_validation() {
        for valid_name in ["j", "mike", "asdjfasdf", "al'i", "o'brien", "n'n"] {
            assert!(is_valid_name(valid_name));
        }
        for invalid_name in ["'ello", "m''ke", "qwert'"] {
            assert!(!is_valid_name(invalid_name));
        }
    }

    #[test]
    fn test_parser() {
        let good_res = parse_names("What is going on? Must be the w'ind.");
        assert_eq!(
            good_res.unwrap(),
            HashSet::from_iter(
                ["hat", "is", "going", "on", "ust", "be", "the", "w'ind"].map(String::from)
            )
        );

        let bad_res_invalid_name = parse_names("'ah', said the fish to the fish.");
        assert!(bad_res_invalid_name.is_err());
        let invalid_name_err = bad_res_invalid_name.unwrap_err();
        assert_eq!(
            invalid_name_err,
            ParseError::InvalidName(String::from("'ah'"))
        );

        let bad_res_dupe_name = parse_names("said the fish to the fish.");
        assert!(bad_res_dupe_name.is_err());
        let dupe_name_err = bad_res_dupe_name.unwrap_err();
        assert_eq!(
            dupe_name_err,
            ParseError::DuplicateName(String::from("the"))
        );
    }

    #[test]
    fn test_tank_from_mask_and_name() {
        let mask = FONT[0];
        let tank = Tank::from_mask_and_name(String::default(), mask).unwrap();
        let expected = Grid::from_vec(
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1],
            4,
        );
        assert_eq!(tank.grid, expected);
    }

    #[test]
    fn test_populate_tanks() {
        let names = HashSet::from([String::from("ab")]);
        let tanks = populate_tanks(names).unwrap();
        assert_eq!(tanks[0].name, String::from("ab"));
        assert_eq!(
            tanks[0].grid,
            Grid::from_vec(
                vec![1, 0, 0, 0, 1, 2, 2, 1, 2, 0, 0, 2, 2, 0, 0, 2, 1, 2, 2, 1],
                4
            )
        );
    }
}
