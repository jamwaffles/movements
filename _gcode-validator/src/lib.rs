use gcode_parser::Block;
use gcode_parser::{tokens::Stopping, GcodeProgram, Token, TokenType};
use rayon::prelude::*;

trait Rule: Sync {
    fn validate(&self, program: &GcodeProgram) -> Result<(), Vec<ValidationError>>;
}

#[derive(Debug)]
pub struct ValidationError<'a> {
    /// The block in which this error occurred
    block: Option<Block<'a>>,

    /// The token(s) in which this error occurred
    tokens: Option<Vec<Token<'a>>>,

    /// Human readable error message
    message: String,
}

impl<'a> ValidationError<'a> {
    pub fn message_only<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            tokens: None,
            block: None,
            message: s.into(),
        }
    }
}

struct ProgramEnd;

impl Rule for ProgramEnd {
    fn validate(&self, program: &GcodeProgram) -> Result<(), Vec<ValidationError>> {
        program
            .token_iter()
            .rev()
            .find(|t| t.token == TokenType::Stopping(Stopping::EndProgram))
            .map(|_t| ())
            .ok_or(vec![ValidationError::message_only(
                "Program must be terminated by M2",
            )])
    }
}

// THINK: How do you validate MDI where there's already a program state? I think I need to write the interpreter first, with a way of feeding blocks into the validator

// Modes: bail on first error, or validate entire program? Some errors should bail (fuck off with "feedrate not set" for every G1 with no feed), some can be checked and collected (program doesn't have an M2)

pub struct GcodeValidator {
    rules: Vec<Box<dyn Rule>>,
}

impl GcodeValidator {
    pub fn new() -> Self {
        Self {
            rules: vec![Box::new(ProgramEnd)],
        }
    }

    pub fn validate(&self, program: &GcodeProgram) -> Result<(), Vec<ValidationError>> {
        self.rules
            .par_iter()
            .map(|rule| rule.validate(&program))
            .collect::<Result<(), Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_program() {
        let program = GcodeProgram::from_str(
            r#"F500
        M2"#,
        )
        .unwrap();

        let validator = GcodeValidator::new();

        validator.validate(&program).unwrap();
    }

    #[test]
    fn no_end_program() {
        let program = GcodeProgram::from_str(r#"F500"#).unwrap();

        let validator = GcodeValidator::new();

        validator.validate(&program).unwrap();
    }
}
