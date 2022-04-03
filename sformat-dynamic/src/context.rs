use crate::{format, Name};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

/// Wraps the string representation of a value.
pub(crate) enum StringRepresentation<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> AsRef<str> for StringRepresentation<'a> {
    fn as_ref(&self) -> &str {
        match self {
            StringRepresentation::Borrowed(str_ref) => str_ref,
            StringRepresentation::Owned(str_owned) => &str_owned[..],
        }
    }
}

/// Sign of a numeric value.
#[derive(Clone, Copy)]
pub(crate) enum Sign {
    Positive,
    Negative,
    Zero,
}

impl Into<u8> for Sign {
    fn into(self) -> u8 {
        match self {
            Sign::Positive => b'+',
            Sign::Negative => b'-',
            Sign::Zero => b'+',
        }
    }
}

/// Wraps a dynamic pointer to a Debug or Display struct.
#[derive(Clone, Copy)]
pub enum DynPointer<'a> {
    Debug(&'a dyn Debug),
    Display(&'a dyn Display),
}

/// Wraps a value with type information.
#[derive(Clone, Copy)]
pub enum TypedValue<'a> {
    Str(&'a str),
    Int(isize),
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    Uint(usize),
    Uint64(u64),
    Uint32(u32),
    Uint16(u16),
    Uint8(u8),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Dyn(DynPointer<'a>),
}

impl<'a> TypedValue<'a> {
    pub(crate) fn string_repr(&self) -> StringRepresentation<'a> {
        match self {
            TypedValue::Str(inner) => StringRepresentation::Borrowed(inner),
            TypedValue::Int(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Int64(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Int32(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Int16(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Int8(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Uint(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Uint64(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Uint32(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Uint16(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Uint8(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Float32(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Float64(num) => StringRepresentation::Owned(num.to_string()),
            TypedValue::Bool(true) => StringRepresentation::Borrowed("true"),
            TypedValue::Bool(false) => StringRepresentation::Borrowed("false"),
            TypedValue::Dyn(DynPointer::Debug(debug)) => {
                StringRepresentation::Owned(format!("{:?}", debug))
            }
            TypedValue::Dyn(DynPointer::Display(display)) => {
                StringRepresentation::Owned(format!("{}", display))
            }
        }
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            TypedValue::Int(_)
                | TypedValue::Int64(_)
                | TypedValue::Int32(_)
                | TypedValue::Int16(_)
                | TypedValue::Int8(_)
                | TypedValue::Uint(_)
                | TypedValue::Uint64(_)
                | TypedValue::Uint32(_)
                | TypedValue::Uint16(_)
                | TypedValue::Uint8(_)
                | TypedValue::Float32(_)
                | TypedValue::Float64(_)
        )
    }

    pub(crate) fn sign(&self) -> Option<Sign> {
        macro_rules! match_signed_int {
            ($e:expr) => {
                match $e.signum() {
                    1 => Some(Sign::Positive),
                    0 => Some(Sign::Zero),
                    -1 => Some(Sign::Negative),
                    _ => unreachable!(),
                }
            };
        }

        macro_rules! match_unsigned_int {
            ($e:expr) => {
                match $e {
                    0 => Some(Sign::Zero),
                    _ => Some(Sign::Positive),
                }
            };
        }

        macro_rules! match_float {
            ($e:expr) => {
                if $e.is_finite() {
                    if $e.is_sign_positive() {
                        Some(Sign::Positive)
                    } else if $e.is_sign_negative() {
                        Some(Sign::Negative)
                    } else {
                        unreachable!()
                    }
                } else {
                    None // NaN, INFINITY
                }
            };
        }

        match self {
            TypedValue::Int(num) => match_signed_int!(num),
            TypedValue::Int64(num) => match_signed_int!(num),
            TypedValue::Int32(num) => match_signed_int!(num),
            TypedValue::Int16(num) => match_signed_int!(num),
            TypedValue::Int8(num) => match_signed_int!(num),
            TypedValue::Uint(num) => match_unsigned_int!(num),
            TypedValue::Uint64(num) => match_unsigned_int!(num),
            TypedValue::Uint32(num) => match_unsigned_int!(num),
            TypedValue::Uint16(num) => match_unsigned_int!(num),
            TypedValue::Uint8(num) => match_unsigned_int!(num),
            TypedValue::Float32(num) => match_float!(num),
            TypedValue::Float64(num) => match_float!(num),
            _ => None,
        }
    }
}

pub trait Context<'ctxt> {
    fn get_variable<'b>(&self, name: Name<'b>) -> Result<TypedValue<'ctxt>, format::Error<'b>>;
}

impl<'ctxt> Context<'ctxt> for HashMap<Name<'_>, TypedValue<'ctxt>> {
    fn get_variable<'b>(&self, name: Name<'b>) -> Result<TypedValue<'ctxt>, format::Error<'b>> {
        self.get(name)
            .copied()
            .ok_or(format::Error::VariableNameError(name))
    }
}
