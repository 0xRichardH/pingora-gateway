use std::ops::Deref;

/// An alias for the `Result` type
pub type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>;

/// Generic wrapper
/// for external types to type From/TryFrom conversions
pub struct W<T>(pub T);

impl<T> Deref for W<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> W<T> {
    pub fn get_wrap(self) -> T {
        self.0
    }
}
