use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaintLevel {
    Trusted,
    UserProvided,
    RetrievedUntrusted,
}
