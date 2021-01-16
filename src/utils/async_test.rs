use futures::executor::block_on;

pub fn run_async_test(f: impl std::future::Future<Output = ()>) {
    block_on(f)
}
