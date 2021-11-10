use std::future::Future;

pub fn consume_with<T>(supplier: impl FnOnce() -> T, consumer: impl Fn(T) -> ()) {
    consumer(supplier());
}

pub async fn apply_with<T, R, S>(supplier: impl FnOnce() -> T, function: impl FnOnce(T) -> R) -> S
where
    R: Future<Output = S>,
{
    function(supplier()).await
}
