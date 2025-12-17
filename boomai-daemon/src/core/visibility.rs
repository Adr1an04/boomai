#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// never expose outside of core
    Private,
    /// admin only (logs, diagnostics)
    Sensitive,
    /// ui safe but not api
    Sanitized,
    /// safe 
    Public,
}

pub trait Sanitizable {
    type Sanitized;
    fn sanitized(&self) -> Self::Sanitized;
}

pub struct Sanitizer;

impl Sanitizer {
    
    pub fn for_ui<T: Sanitizable>(data: &T) -> T::Sanitized {
        data.sanitized()
    }

    pub fn diagnostics_enabled() -> bool {
        // read from config or env var in future
        false
    }
}
