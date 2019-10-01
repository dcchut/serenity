use futures::executor::block_on;

pub fn run_async_test(f : impl std::future::Future<Output = ()>) {
    block_on(f)
}

pub trait AsyncFrom<T: Send>: Sized {
    #[doc = " Performs the conversion."]
    fn async_from<'async_trait>(
        __arg0: T,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Self> + core::marker::Send + 'async_trait>,
    >
        where
            Self: 'async_trait;
}
pub trait AsyncInto<T: Send>: Sized {
    #[doc = " Performs the conversion."]
    fn async_into<'async_trait>(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = T> + core::marker::Send + 'async_trait>,
    >
        where
            Self: 'async_trait,
            T: 'async_trait; // Need extra bound here
}

impl<T: Send, U: Send> AsyncInto<U> for T
    where
        U: AsyncFrom<T>,
{
    fn async_into<'async_trait>(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = U> + core::marker::Send + 'async_trait>,
    >
        where
            Self: 'async_trait,
            U: 'async_trait
    {
        #[allow(clippy::used_underscore_binding)]
        async fn __async_into<T: Send, U: Send>(_self: T) -> U
            where
                (): Sized,
                U: AsyncFrom<T>,
        {
            U::async_from(_self).await
        }
        Box::pin(__async_into::<T, U>(self))
    }
}