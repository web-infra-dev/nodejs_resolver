use crate::{Error, Info, ResolveResult};

#[derive(Debug)]
pub enum State {
    Success(ResolveResult),
    Resolving(Info),
    /// return error directly
    Error(Error),
    /// forEachBail
    Failed(Info),
}

impl State {
    pub fn then<F: FnOnce(Info) -> State>(self, op: F) -> Self {
        match self {
            State::Resolving(info) => op(info),
            _ => self,
        }
    }

    pub fn map_success<F: FnOnce(Info) -> State>(self, op: F) -> Self {
        match self {
            State::Success(ResolveResult::Info(info)) => op(info),
            _ => self,
        }
    }

    pub fn map_failed<F: FnOnce(Info) -> State>(self, op: F) -> Self {
        match self {
            State::Failed(info) => op(info),
            _ => self,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, State::Success(_) | State::Error(_))
    }
}
