use std::result;

error_chain!{}

pub trait ErrorChainable<T> {
    fn error(self, e: impl Into<Error>) -> Result<T>;
}

impl<T> ErrorChainable<T> for Option<T> {
    fn error(self, e: impl Into<Error>) -> Result<T> {
        self.ok_or_else(|| e.into())
    }
}

impl<T, E> ErrorChainable<T> for result::Result<T, E> {
    fn error(self, e: impl Into<Error>) -> Result<T> {
        self.map_err(|_| e.into())
    }
}
