/// Trait for types that can be sanitized for UI exposure
pub trait Sanitizable {
    type Sanitized;
    fn sanitized(&self) -> Self::Sanitized;
}
