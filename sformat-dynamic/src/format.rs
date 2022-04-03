use crate::{
    context::{Sign, TypedValue},
    Name,
};
use std::{
    io::{self, Write},
    ops::Range,
};

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("error writing literal: {0}")]
    WriteLiteralError(io::Error),

    #[error("error writing variable({0}: {1}")]
    WriteVariableError(Name<'a>, io::Error),

    #[error("error finding name: {0}")]
    VariableNameError(Name<'a>),

    #[error("variable ({0}) had incorrect type")]
    VariableTypeError(Name<'a>),
}

pub const ZERO_FILL: Fill = Fill::new(Some('0'), Alignment::Right);
pub const DEFAULT_FILL: Fill = Fill::new(Some(' '), Alignment::Left);

/// The `SignFlag` can be specified to always print the sign of a number.
///
/// See [str::fmt documentation about flags](https://doc.rust-lang.org/std/fmt/#sign0).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignFlag {
    Plus,
    Minus, // Unused
}

impl SignFlag {
    /// Gets the sign for a numeric value if `SignFlag::Plus` is set.
    ///
    /// See [str::fmt documentation about flags](https://doc.rust-lang.org/std/fmt/#sign0).
    ///
    /// # Arguments
    ///
    /// * `val`     - The value being written to the writeable instance.
    fn get_sign_for_value(&self, val: TypedValue<'_>) -> Option<Sign> {
        match self {
            SignFlag::Plus => val.sign(),
            SignFlag::Minus => None, // Unused
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Flags {
    sign: Option<SignFlag>,

    // Zero flag. If specified, the format string is number aware.
    zero: Option<()>,
}

impl Flags {
    pub const fn new(sign: Option<SignFlag>, zero: Option<()>) -> Flags {
        Flags { sign, zero }
    }

    fn is_number_aware(&self) -> bool {
        self.zero.is_some()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Format {
    fill: Option<Fill>,
    flags: Flags,
    width: Option<u32>,
    precision: Option<u32>,
}

impl Format {
    pub const fn new(
        fill: Option<Fill>,
        flags: Flags,
        width: Option<u32>,
        precision: Option<u32>,
    ) -> Format {
        Format {
            fill,
            flags,
            width,
            precision,
        }
    }

    fn get_fill(&self, val: &TypedValue<'_>) -> Fill {
        if self.flags.is_number_aware() && val.is_numeric() {
            ZERO_FILL
        } else {
            self.fill.unwrap_or(DEFAULT_FILL)
        }
    }

    fn write_sign<W>(&self, sign: Sign, write: &mut W) -> Result<Option<Sign>, io::Error>
    where
        W: Write,
    {
        match sign {
            sign @ Sign::Positive | sign @ Sign::Zero => {
                write.write_all(&[sign.into()])?;

                Ok(Some(sign))
            }
            sign @ Sign::Negative if self.flags.is_number_aware() => {
                write.write_all(&[sign.into()])?;

                Ok(Some(sign))
            }
            // Negative Flag is written as a part of the string
            // representation already.
            _ => Ok(None),
        }
    }

    pub fn write_formatted<'a, W>(
        &self,
        val: TypedValue<'a>,
        write: &mut W,
    ) -> Result<(), io::Error>
    where
        W: Write,
    {
        let write_str = val.string_repr();
        let mut write_str = write_str.as_ref();
        let sign = self
            .flags
            .sign
            .and_then(|sign_flag| sign_flag.get_sign_for_value(val));

        match self.width {
            // There is a width specified, but the string that is being written
            // is actually larger than what is specified.
            Some(width) if write_str.len() > width as usize => {
                if let Some(sign) = sign {
                    self.write_sign(sign, write)?;
                }

                write.write_all(write_str.as_bytes())?;
            }
            // No width is specified.
            None => {
                if let Some(sign) = sign {
                    self.write_sign(sign, write)?;
                }

                write.write_all(write_str.as_bytes())?;
            }
            // A width is specified. Implicit: The string that is being written
            // is smaller than the specified width.
            Some(width) => {
                let mut width = width as usize;
                let fill = self.get_fill(&val);
                if self.flags.is_number_aware() {
                    // For a number aware (zero-flag) format, the sign has to be written
                    // first. This means that for a negative number, the string that
                    // gets written has to be truncated slightly.
                    match sign
                        .map(|sign| self.write_sign(sign, write))
                        .transpose()?
                        .flatten()
                    {
                        Some(Sign::Positive) | Some(Sign::Zero) => {
                            width -= 1;
                        }
                        Some(Sign::Negative) => {
                            width -= 1;
                            write_str = &write_str[1..];
                        }
                        _ => (),
                    }
                } else {
                    // If the zero-flag is not specified, writing the "+" sign is
                    // deferred until after the filler is written, but it should still
                    // be considered when the left filler.
                    match sign {
                        Some(Sign::Positive) | Some(Sign::Zero) => {
                            width -= 1;
                        }
                        _ => (),
                    }
                }

                width -= fill.write_left_filler(write_str, width, write)?;

                if !self.flags.is_number_aware() {
                    sign.map(|sign| self.write_sign(sign, write)).transpose()?;
                }

                write.write_all(write_str.as_bytes())?;

                width -= write_str.len();

                fill.write_right_filler(width, write)?;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Fill {
    fill_char: Option<char>,
    alignment: Alignment,
}

impl Fill {
    pub const fn new(fill_char: Option<char>, alignment: Alignment) -> Fill {
        Fill {
            fill_char,
            alignment,
        }
    }

    fn get_fill_char_or_default(&self) -> char {
        self.fill_char.unwrap_or(' ')
    }

    fn write_filler<W>(&self, range: Range<usize>, write: &mut W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let mut buffer = [0; 4];
        let bytes_to_write = range.len();
        for _ in range {
            let encoded = self.get_fill_char_or_default().encode_utf8(&mut buffer);

            write.write_all(encoded.as_bytes())?;
        }

        Ok(bytes_to_write)
    }

    fn write_left_filler<W>(
        &self,
        val: &str,
        width: usize,
        write: &mut W,
    ) -> Result<usize, io::Error>
    where
        W: Write,
    {
        match self.alignment {
            // NOOP
            Alignment::Left => Ok(0),
            Alignment::Center => {
                let left_pad = (width - val.len()) / 2;

                self.write_filler(0..left_pad, write)
            }
            Alignment::Right => self.write_filler(0..width - val.len(), write),
        }
    }

    fn write_right_filler<W>(&self, width: usize, write: &mut W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        match self.alignment {
            Alignment::Left | Alignment::Center => self.write_filler(0..width, write),
            // NOOP
            Alignment::Right => Ok(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}
