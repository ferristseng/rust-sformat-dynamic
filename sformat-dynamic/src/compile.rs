use crate::{
    context::Context,
    format::{self, Alignment, Fill, Flags, Format, SignFlag},
    token::Token,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{anychar, char, satisfy, u32},
    combinator::{eof, map, opt, recognize, value},
    error::{ErrorKind, ParseError},
    multi::many_till,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use std::io::Write;
use unicode_xid::UnicodeXID;

/// Error compiling a format string.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("error parsing format string: {0}")]
    ParseError(#[from] nom::Err<(String, ErrorKind)>),
}

/// Parses '<', '^', or '>'.
fn alignment_parser<'a, Error>(input: &'a str) -> IResult<&'a str, Alignment, Error>
where
    Error: ParseError<&'a str>,
{
    alt((
        value(Alignment::Left, char('<')),
        value(Alignment::Center, char('^')),
        value(Alignment::Right, char('>')),
    ))(input)
}

fn flags_parser<'a, Error>(input: &'a str) -> IResult<&'a str, Flags, Error>
where
    Error: ParseError<&'a str>,
{
    map(
        tuple((
            opt(alt((
                value(SignFlag::Plus, char('+')),
                value(SignFlag::Minus, char('-')),
            ))),
            opt(value((), char('0'))),
        )),
        |(sign, zero)| Flags::new(sign, zero),
    )(input)
}

fn precision_parser<'a, Error>(input: &'a str) -> IResult<&'a str, u32, Error>
where
    Error: ParseError<&'a str>,
{
    preceded(char('.'), r#u32)(input)
}

/// Parses a Rust identifier.
///
/// See documentation on Rust identifiers: https://doc.rust-lang.org/reference/identifiers.html
fn rust_identifier_parser<'a, Error>(input: &'a str) -> IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    recognize(pair(
        satisfy(|c| c.is_xid_start()),
        take_while(|c: char| c == '_' || c.is_xid_continue()),
    ))(input)
}

/// Parses a format spec.
///
/// Format spec is described here: https://doc.rust-lang.org/std/fmt/
fn format_parser<'a, Error>(input: &'a str) -> IResult<&'a str, Format, Error>
where
    Error: ParseError<&'a str>,
{
    let fill_spec = alt((
        map(
            tuple((anychar, alignment_parser)),
            |(fill_char, alignment)| Fill::new(Some(fill_char), alignment),
        ),
        map(alignment_parser, |alignment| Fill::new(None, alignment)),
    ));
    let width_spec = r#u32;

    map(
        preceded(
            char(':'),
            tuple((
                opt(fill_spec),
                flags_parser,
                opt(width_spec),
                opt(precision_parser),
            )),
        ),
        |(fill, flags, width, precision)| Format::new(fill, flags, width, precision),
    )(input)
}

/// Compiles a format string.
pub fn compile(format_str: &'_ str) -> Result<CompiledFormat<'_>, CompileError> {
    let (_all, (tokens, _rest)) = many_till(
        alt((
            // Escape Left Curly Brace
            map(tag("{{"), Token::Literal),
            // Identifier
            map(
                delimited(
                    char('{'),
                    tuple((rust_identifier_parser, opt(format_parser))),
                    char('}'),
                ),
                |(identifier, format)| Token::Variable(identifier, format),
            ),
            // Literal
            map(take_while1(|c: char| c != '{'), Token::Literal),
        )),
        eof,
    )(format_str)
    .map_err(nom::Err::<(&str, ErrorKind)>::to_owned)?;

    Ok(CompiledFormat { ast: tokens })
}

#[derive(Debug)]
pub struct CompiledFormat<'format> {
    ast: Vec<Token<'format>>,
}

impl<'format> CompiledFormat<'format> {
    pub fn format<'ctxt, W, C>(
        &self,
        write: &mut W,
        context: &'ctxt C,
    ) -> Result<(), format::Error<'ctxt>>
    where
        W: Write,
        C: Context<'ctxt>,
        'format: 'ctxt,
    {
        for token in self.ast.iter() {
            token.write_token(write, context)?;
        }

        Ok(())
    }

    pub fn format_str<'ctxt, C>(&self, context: &'ctxt C) -> Result<String, format::Error<'ctxt>>
    where
        C: Context<'ctxt>,
        'format: 'ctxt,
    {
        let mut formatted = Vec::new();

        self.format(&mut formatted, context)?;

        Ok(String::from_utf8(formatted).expect("expected utf-8 encoded string"))
    }

    #[cfg(test)]
    pub fn into_ast(self) -> Vec<Token<'format>> {
        self.ast
    }
}

impl<'format> TryFrom<&'format str> for CompiledFormat<'format> {
    type Error = CompileError;

    fn try_from(format_str: &'format str) -> Result<Self, Self::Error> {
        compile(format_str)
    }
}

#[cfg(test)]
mod tests {
    use super::compile;
    use crate::{
        context::{DynPointer, TypedValue},
        format::{self, Alignment, Fill, Flags, Format, SignFlag},
        token::Token,
    };
    use std::collections::HashMap;

    macro_rules! compile_test {
        (
            [$test_name:ident]
            COMPILE $input:literal
            TO AST $output:expr
        ) => {
            #[test]
            fn $test_name() {
                let fmt = compile($input);

                assert!(fmt.is_ok(), "Err: {:?}", fmt);
                assert_eq!(fmt.unwrap().into_ast(), $output);
            }
        };
    }

    compile_test! {
        [test_compile_content]
        COMPILE "hello {test} this is {ferris}"
        TO AST vec![
            Token::Literal("hello "),
            Token::Variable("test", None),
            Token::Literal(" this is "),
            Token::Variable("ferris", None)
        ]
    }

    compile_test! {
        [test_compile_empty]
        COMPILE ""
        TO AST vec![]
    }

    compile_test! {
        [test_compile_literals]
        COMPILE "hello only literals"
        TO AST vec![
            Token::Literal("hello only literals")
        ]
    }

    compile_test! {
        [test_compile_escaped_left_brace]
        COMPILE "{{ {{ {{"
        TO AST vec![
            Token::Literal("{{"),
            Token::Literal(" "),
            Token::Literal("{{"),
            Token::Literal(" "),
            Token::Literal("{{")
        ]
    }

    compile_test! {
        [test_compile_unicode]
        COMPILE "我的名字是{名字}"
        TO AST vec![
            Token::Literal("我的名字是"),
            Token::Variable("名字", None)
        ]
    }

    compile_test! {
        [test_compile_fill]
        COMPILE "{number:*>5}"
        TO AST vec![
            Token::Variable(
                "number",
                Some(
                    Format::new(
                        Some(Fill::new(Some('*'), Alignment::Right)),
                        Flags::default(),
                        Some(5u32),
                        None
                    )
                )
            )
        ]
    }

    compile_test! {
        [test_compile_fill_default]
        COMPILE "{test:^200}"
        TO AST vec![
            Token::Variable(
                "test",
                Some(
                    Format::new(
                        Some(Fill::new(None, Alignment::Center)),
                        Flags::default(),
                        Some(200u32),
                        None
                    )
                )
            )
        ]
    }

    compile_test! {
        [test_compile_flags_default]
        COMPILE "{test:+056}"
        TO AST vec![
            Token::Variable(
                "test",
                Some(
                    Format::new(
                        None,
                        Flags::new(Some(SignFlag::Plus), Some(())),
                        Some(56u32),
                        None
                    )
                )
            )
        ]
    }

    compile_test! {
        [test_compile_precision_number]
        COMPILE "{test:.15}"
        TO AST vec![
            Token::Variable(
                "test",
                Some(
                    Format::new(
                        None,
                        Flags::default(),
                        None,
                        Some(15)
                    )
                )
            )
        ]
    }

    macro_rules! format_test {
        (
            [$test_name:ident]
            FORMAT $input:literal
            WITH CTXT $context:expr;
            EQUALS $output:expr;
        ) => {
            #[test]
            fn $test_name() {
                let fmt = compile($input);

                assert!(fmt.is_ok(), "{:?}", fmt);

                let context = $context;
                let formatted = fmt.unwrap().format_str(&context);

                assert!(formatted.is_ok(), "{:?}", formatted);

                println!(
                    "{:64} >>> ACTUAL: {} | EXPECTED: {}",
                    stringify!($test_name),
                    formatted.as_ref().unwrap(),
                    $output
                );

                assert_eq!(formatted.unwrap(), $output);
            }
        };
        (
            [$test_name:ident]
            FORMAT $input:literal
            WITH CTXT $context:expr;
            FAILS WITH $error:pat
        ) => {
            #[test]
            fn $test_name() {
                let fmt = compile($input);

                assert!(fmt.is_ok());

                let context = $context;
                let formatted = fmt.unwrap().format_str(&context);

                assert!(formatted.is_err());
                assert!(matches!(formatted.err().unwrap(), $error));
            }
        };
    }

    format_test! {
        [test_format_literal]
        FORMAT "hello this is a test"
        WITH CTXT HashMap::new();
        EQUALS "hello this is a test";
    }

    format_test! {
        [test_format_single_variable]
        FORMAT "hello this is a {test}"
        WITH CTXT HashMap::from([
            ("test", TypedValue::Str("blergh"))
        ]);
        EQUALS "hello this is a blergh";
    }

    format_test! {
        [test_format_width_but_no_fill]
        FORMAT "Hello {name:10}!"
        WITH CTXT HashMap::from([
            ("name", TypedValue::Str("Ferris"))
        ]);
        EQUALS format!("Hello {:10}!", "Ferris");
    }

    format_test! {
        [test_format_multiple_variables]
        FORMAT "hello this is a {test}. my name is {name}. i am {age}."
        WITH CTXT HashMap::from([
            ("test", TypedValue::Str("SERIOUS TEST")),
            ("name", TypedValue::Str("Ferris")),
            ("age", TypedValue::Uint(100))
        ]);
        EQUALS "hello this is a SERIOUS TEST. my name is Ferris. i am 100.";
    }

    format_test! {
        [test_format_multiple_floats]
        FORMAT "{float64} {float32} {infinity} {neg_infinity} {nan}"
        WITH CTXT HashMap::from([
            ("float64", TypedValue::Float64(1938917398.13817f64)),
            ("float32", TypedValue::Float32(-15984.12351f32)),
            ("infinity", TypedValue::Float64(f64::INFINITY)),
            ("neg_infinity", TypedValue::Float32(f32::NEG_INFINITY)),
            ("nan", TypedValue::Float64(f64::NAN))
        ]);
        EQUALS format!(
            "{} {} {} {} {}",
            1938917398.13817f64,
            -15984.12351f32,
            f64::INFINITY,
            f32::NEG_INFINITY,
            f64::NAN
        );
    }

    format_test! {
        [test_format_extra_context]
        FORMAT "hello this is a {severity} test"
        WITH CTXT HashMap::from([
            ("severity", TypedValue::Str("SERIOUS")),
            ("level", TypedValue::Uint32(128u32))
        ]);
        EQUALS "hello this is a SERIOUS test";
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct TestStruct {
        name: &'static str,
        num: usize,
    }

    const TEST_STRUCT: TestStruct = TestStruct {
        name: "test",
        num: 1000,
    };

    format_test! {
        [test_format_debug_struct]
        FORMAT "{struct}"
        WITH CTXT HashMap::from([
            ("struct", TypedValue::Dyn(DynPointer::Debug(&TEST_STRUCT)))
        ]);
        EQUALS "TestStruct { name: \"test\", num: 1000 }";
    }

    format_test! {
        [test_format_fill_right_align]
        FORMAT "{number:*>10}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint32(0))
        ]);
        EQUALS format!("{:*>10}", 0);
    }

    format_test! {
        [test_format_fill_center_align_even]
        FORMAT "{number:0^9}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int8(8))
        ]);
        EQUALS format!("{:0^9}", 8);
    }

    format_test! {
        [test_format_fill_center_align_uneven]
        FORMAT "{number:0^8}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint32(123))
        ]);
        EQUALS format!("{:0^8}", 123u32);
    }

    format_test! {
        [test_format_fill_center_align_too_long]
        FORMAT "{s:@^4}"
        WITH CTXT HashMap::from([
            ("s", TypedValue::Str("hello world"))
        ]);
        EQUALS format!("{:@^4}", "hello world");
    }

    format_test! {
        [test_format_fill_default_char]
        FORMAT "{bool:<7}"
        WITH CTXT HashMap::from([
            ("bool", TypedValue::Bool(false))
        ]);
        EQUALS format!("{:<#7}", false);
    }

    format_test! {
        [test_format_fill_numeric_left_align]
        FORMAT "{number:$<20}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int32(-1000))
        ]);
        EQUALS format!("{:$<20}", -1000i32);
    }

    format_test! {
        [test_format_fill_numeric_right_align]
        FORMAT "{number:?>30}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int32(-1000))
        ]);
        EQUALS format!("{:?>30}", -1000i32);
    }

    format_test! {
        [test_format_fill_left_align_signed_number]
        FORMAT "{number:X<+12}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Float32(-12.12f32))
        ]);
        EQUALS format!("{:X<+12}", -12.12f32);
    }

    format_test! {
        [test_format_fill_right_align_signed_number]
        FORMAT "{number:X>+18}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Float64(-1.125134f64))
        ]);
        EQUALS format!("{:X>+18}", -1.125134f64);
    }

    format_test! {
        [test_format_fill_left_align_unsigned_number]
        FORMAT "{number:.<+15}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint(319))
        ]);
        EQUALS format!("{:.<+15}", 319);
    }

    format_test! {
        [test_format_fill_right_align_unsigned_number]
        FORMAT "{number:.>+23}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint(32))
        ]);
        EQUALS format!("{:.>+23}", 32);
    }

    format_test! {
        [test_format_fill_center_align_unsigned_number]
        FORMAT "{number:.^+9}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint(9))
        ]);
        EQUALS format!("{:.^+9}", 9);
    }

    format_test! {
        [test_format_sign_positive_number]
        FORMAT "{number:+}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int64(128i64))
        ]);
        EQUALS format!("{:+}", 128i64);
    }

    format_test! {
        [test_format_sign_zero]
        FORMAT "{number:+}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint8(0))
        ]);
        EQUALS format!("{:+}", 0);
    }

    format_test! {
        [test_format_sign_negative_number]
        FORMAT "{number:+}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int32(i32::MIN))
        ]);
        EQUALS format!("{:+}", i32::MIN);
    }

    format_test! {
        [test_format_sign_float_nan]
        FORMAT "{number:+}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Float32(f32::NAN))
        ]);
        EQUALS format!("{:+}", f32::NAN);
    }

    format_test! {
        [test_format_sign_float_infinity]
        FORMAT "{number:+}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Float64(f64::NEG_INFINITY))
        ]);
        EQUALS format!("{:+}", f64::NEG_INFINITY);
    }

    format_test! {
        [test_format_fill_and_zero_flag_specified_signed_number]
        FORMAT "{number:.^+012}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Int(-194))
        ]);
        EQUALS format!("{:.^+012}", -194);
    }

    format_test! {
        [test_format_fill_and_zero_flag_specified_unsigned_number]
        FORMAT "{number:.<+09}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Uint(129))
        ]);
        EQUALS format!("{:.<+09}", 129);
    }

    format_test! {
        [test_format_fill_and_zero_flag_specified_non_numeric]
        FORMAT "{number:.<+021}"
        WITH CTXT HashMap::from([
            ("number", TypedValue::Str("1000"))
        ]);
        EQUALS format!("{:.<+021}", "1000");
    }

    format_test! {
        [test_format_zero_float]
        FORMAT "{float:.15}"
        WITH CTXT HashMap::from([
            ("float", TypedValue::Float64(0.0))
        ]);
        EQUALS format!("{:.15}", 0.0);
    }

    format_test! {
        [test_format_non_zero_float]
        FORMAT "{float:.15}"
        WITH CTXT HashMap::from([
            ("float", TypedValue::Float64(10.1562))
        ]);
        EQUALS format!("{:.15}", 10.1562);
    }

    format_test! {
        [test_format_non_zero_usize]
        FORMAT "{usize:.15}"
        WITH CTXT HashMap::from([
            ("usize", TypedValue::Uint(128))
        ]);
        EQUALS format!("{:.15}", 128);
    }

    format_test! {
        [test_format_fill_smaller_than_precision]
        FORMAT "{float:*<4.15}"
        WITH CTXT HashMap::from([
            ("float", TypedValue::Float64(1.123456))
        ]);
        EQUALS format!("{:*<4.15}", 1.123456);
    }

    format_test! {
        [test_format_fill_larger_than_precision]
        FORMAT "{float:*<20.11}"
        WITH CTXT HashMap::from([
            ("float", TypedValue::Float64(283.1239))
        ]);
        EQUALS format!("{:*<20.11}", 283.1239);
    }

    format_test! {
        [test_format_missing_variable]
        FORMAT "hello this is a {severity} test"
        WITH CTXT HashMap::new();
        FAILS WITH format::Error::VariableNameError("severity")
    }
}
