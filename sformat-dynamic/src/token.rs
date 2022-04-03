use crate::{
    context::Context,
    format::{self, Format},
    Name,
};
use std::io::Write;

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'format> {
    Literal(&'format str),
    Variable(Name<'format>, Option<Format>),
}

impl<'format> Token<'format> {
    pub(crate) fn write_token<'b, W, C>(
        &self,
        write: &mut W,
        context: &'b C,
    ) -> Result<(), format::Error<'b>>
    where
        W: Write,
        C: Context<'b>,
        'format: 'b,
    {
        match self {
            Token::Literal(lit) => write
                .write_all(lit.as_bytes())
                .map_err(format::Error::WriteLiteralError),
            Token::Variable(name, None) => {
                let val = context.get_variable(name)?.string_repr();

                write
                    .write_all(val.as_ref().as_bytes())
                    .map_err(|err| format::Error::WriteVariableError(name, err))
            }
            Token::Variable(name, Some(format)) => {
                let val = context.get_variable(name)?;

                format
                    .write_formatted(val, write)
                    .map_err(|err| format::Error::WriteVariableError(name, err))
            }
        }
    }
}
