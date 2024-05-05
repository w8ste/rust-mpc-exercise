pub use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CircuitError {
    ParsingError(String),
    ParsingHeaderInformationError(usize, usize),
    ParsingNovError(usize, usize),
    ParsingNivError(usize, usize),
    EmptyLineMissingError,
    NotAGateError(String),
    WrongGateAmount(usize, usize),
}

impl Error for CircuitError {}

impl Display for CircuitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitError::ParsingError(s) => {
                write!(f, "Parsing failed, due to {}", s)
            }

            CircuitError::ParsingNovError(expected, actual) => {
                write!(
                    f,
                    "An error has occured, whilst parsing nov. Expected {} argument(s), but got {}.",
                    expected, actual
                )
            }
            CircuitError::ParsingNivError(expected, actual) => {
                write!(
                    f,
                    "An error has occured, whilst parsing niv. Expected {} argument(s), but got {}.",
                    expected, actual
                )
            }
            CircuitError::ParsingHeaderInformationError(expected, actual) => {
                write!(
                    f,
                    "An error has occured, whilst parsing the first line. Expected {} argument(s), but got {}.",
                    expected, actual
                )
            }
            CircuitError::EmptyLineMissingError => {
                write!(f, "Expected an empty line")
            }
            CircuitError::NotAGateError(g) => {
                write!(f, "{} is not a valid gate.", g)
            }
            CircuitError::WrongGateAmount(expected, actual) => {
                write!(
                    f,
                    "Wrong amount of Gates. Expected: {}, actually: {}",
                    expected, actual
                )
            }
        }
    }
}
