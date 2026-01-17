//! The module for working with positions in a table. The main type representing some position is
//! [`CellPos`]. It has 2 integer coordinates: x and
//! y (alternatively column and row). It's used to get and set values of the tables.
//!
//! It's represented by a string in the following format:
//! - First, there's the x coordinate. It's represented like a base-26 number with [A-Z] as the
//!   digits, A being the 0. For example, 0 -> A, 1 -> B, 25 -> Z, 26 -> BA, etc.
//! - After the x coordinate without any separator there's the y coordinate written as a decimal.
//!
//! Examples: (0, 0) -> A0, (1, 1) -> B1, (28, 130) -> BC130. Conversions are done using Display
//! and FromStr traits.
//!
//! ```
//! # use bight::table::CellPos;
//! # use std::str::FromStr;
//! # fn main() {
//!     let cell_pos = CellPos::from_str("BC130").unwrap();
//!     assert_eq!(cell_pos, CellPos::from((28, 130)));
//!     let string = format!("{cell_pos}");
//!     assert_eq!(string, "BC130");
//!     assert!(CellPos::from_str("invalid string").is_err());
//! # }
//!
//! ```
//!
//! It can be converted from Lua from a string, 2 non-negative numbers, or a table with x, col, column or 1st element for x coordinate and y, row, or 2nd element for y coordinate

use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

use rkyv::{Archive, Deserialize, Serialize};
#[derive(Clone, Copy, Hash, PartialEq, Eq, Default, Archive, Serialize, Deserialize)]
#[rkyv(derive(PartialEq, Eq, Hash))]
/// A type representing some position in a table. See module level docs for more info.
pub struct CellPos {
    pub x: isize,
    pub y: isize,
}

impl Debug for CellPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<(isize, isize)> for CellPos {
    fn from(value: (isize, isize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

/// Error of converting a &str into a CellPos
#[derive(Debug, thiserror::Error)]
pub enum CellPosParseError {
    #[error("CellPos str contained an invalid digit")]
    InvalidDidit,
}

const LETTER_BASE: u32 = 26;
impl FromStr for CellPos {
    type Err = CellPosParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letters = s
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase());

        let mut x = 0usize;
        for l in letters {
            x *= LETTER_BASE as usize;
            x += l
                .to_digit(LETTER_BASE + 10)
                .expect("Only letters can be in letters") as usize
                - 10;
        }

        let numbers = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .take_while(|c| c.is_ascii_digit());

        let mut y = 0usize;
        for n in numbers {
            y *= 10;
            y += n.to_digit(10).expect("Only digits can be in numbers") as usize;
        }

        let left = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .skip_while(|c| c.is_ascii_digit());

        if left.count() > 0 {
            Err(CellPosParseError::InvalidDidit)
        } else {
            Ok((x.try_into().unwrap(), y.try_into().unwrap()).into())
        }
    }
}

impl Display for CellPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.x;
        if x == 0 {
            write!(f, "A")?;
        }

        let mut chars = Vec::new();
        let mut x: usize = if x < 0 {
            chars.push('n');
            (-x).try_into().unwrap()
        } else {
            x.try_into().unwrap()
        };

        while x > 0 {
            let digit = x % LETTER_BASE as usize;
            let c = char::from_digit(digit as u32 + 10, LETTER_BASE + 10)
                .expect("digit is always less that LETTER_BASE")
                .to_ascii_uppercase();
            chars.push(c);
            x /= LETTER_BASE as usize;
        }
        let y = self.y;
        let y: usize = if y < 0 {
            chars.push('n');
            (-y).try_into().unwrap()
        } else {
            y.try_into().unwrap()
        };

        for c in chars.into_iter().rev() {
            write!(f, "{c}")?;
        }
        write!(f, "{}", y)?;

        Ok(())
    }
}

impl mlua::FromLuaMulti for CellPos {
    fn from_lua_multi(values: mlua::MultiValue, _lua: &mlua::Lua) -> mlua::Result<Self> {
        fn try_lua_to_isize(value: &mlua::Value) -> Option<isize> {
            Some(match value {
                mlua::Value::Integer(x) => (*x).try_into().unwrap(),
                mlua::Value::Number(x) if x.is_normal() => *x as isize,
                _ => return None,
            })
        }

        const ERROR_MESSAGE: &str = "CellPos can be created from a string in format [A-Za-z]+[0-9]+, 2 non-negative numbers, or a table with x, col, column or 1st element for x coordinate and y, row, or 2nd element for y coordinate";
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellPos".into(),
            message: Some(ERROR_MESSAGE.into()),
        });

        let pos = match values.len() {
            0 => return err,
            1 => {
                let value = values.into_iter().next().unwrap();
                match value {
                    mlua::Value::Table(t) => {
                        let Ok(x) = t
                            .get("x")
                            .or_else(|_| t.get("col"))
                            .or_else(|_| t.get("column"))
                            .or_else(|_| t.get(1))
                        else {
                            log::trace!("could find x value");
                            return err;
                        };

                        let Ok(y) = t.get("y").or_else(|_| t.get("row")).or_else(|_| t.get(2))
                        else {
                            log::trace!("could find y value");
                            return err;
                        };
                        CellPos::from((x, y))
                    }
                    mlua::Value::String(s) => {
                        let Ok(pos) = s.to_str() else { return err };
                        let Ok(pos) = pos.parse::<CellPos>() else {
                            return err;
                        };
                        pos
                    }
                    _ => return err,
                }
            }
            2.. => {
                let mut iter = values.into_iter();
                let (x, y) = (iter.next().unwrap(), iter.next().unwrap());
                let (Some(x), Some(y)) = (try_lua_to_isize(&x), try_lua_to_isize(&y)) else {
                    return err;
                };
                CellPos::from((x, y))
            }
        };

        Ok(pos)
    }
}
