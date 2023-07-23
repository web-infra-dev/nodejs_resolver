use crate::{Error, Info, ResolveResult};

#[derive(Debug)]
pub enum State {
    Success(ResolveResult<Info>),
    Resolving(Info),
    /// return error directly
    Error(Error),
    /// forEachBail
    Failed(Info),
}

impl State {
    pub fn is_resolving(&self) -> bool {
        matches!(self, State::Resolving(_))
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, State::Success(_) | State::Error(_))
    }
}
