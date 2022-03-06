use crate::{Endpoint, Error, Middleware, Request};
/// Middleware for add any data to request.
pub struct AddData<T> {
    value: T,
}

impl<T: Clone + Send + Sync + 'static> AddData<T> {
    /// Create new `AddData` middleware with any value.
    pub fn new(value: T) -> Self {
        AddData { value }
    }
}

impl<E, T> Middleware<E> for AddData<T>
where
    E: Endpoint,
    T: Clone + Send + Sync + 'static,
{
    type Output = AddDataEndpoint<E, T>;

    fn transform(&self, ep: E) -> Self::Output {
        AddDataEndpoint {
            inner: ep,
            value: self.value.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AddDataEndpoint<E, T> {
    inner: E,
    value: T,
}

impl<E, T> Endpoint for AddDataEndpoint<E, T>
where
    E: Endpoint,
    T: Clone + Send + Sync + 'static,
{
    type Output = E::Output;

    fn call(&self, mut req: Request) -> Result<Self::Output, Error> {
        req.extensions_mut().insert(self.value.clone());
        self.inner.call(req)
    }
}
