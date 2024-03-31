use std::ops::Deref;

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
