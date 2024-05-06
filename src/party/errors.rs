use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::{RecvError, SendError};

#[derive(Debug)]
pub enum PartyError<'a> {
    ThreadTransmissionError,
    ThreadSendingError,
    ThreadReceivingError,
    WireNotSetError(usize),
    PError(Box<dyn Error + 'a>),
}

impl<'a> Display for PartyError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PartyError::ThreadTransmissionError => {
                write!(f, "Error, whilst Transmissioning Data between Threads")
            }
            PartyError::ThreadSendingError => {
                write!(f, "Error, whilst Transmissioning Data between Threads")
            }
            PartyError::ThreadReceivingError => {
                write!(f, "Error, whilst Transmissioning Data between Threads")
            }
            PartyError::WireNotSetError(wire) => {
                write!(f, "Wire {} has not been set yet", wire)
            }

            PartyError::PError(e) => write!(f, "ProtocolError! {}", *e),
        }
    }
}

impl<'a, T: 'a> From<SendError<T>> for PartyError<'a> {
    fn from(value: SendError<T>) -> Self {
        Self::PError(Box::new(value))
    }
}

impl<'a> From<RecvError> for PartyError<'a> {
    fn from(value: RecvError) -> Self {
        Self::PError(Box::new(value))
    }
}
impl<'a> Error for PartyError<'a> {}
