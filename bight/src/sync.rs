use std::sync::LazyLock;

#[cfg(feature = "sync")]
pub type Rc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "sync"))]
pub type Rc<T> = std::rc::Rc<T>;

pub type RcStr = Rc<str>;

#[cfg(feature = "multi-thread")]
static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[cfg(not(feature = "multi-thread"))]
static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[cfg(feature = "multi-thread")]
use std::pin::Pin;

#[cfg(feature = "multi-thread")]
///
/// Removes lifetime from the future.
///
/// # Safety
/// The future must not be used longer than its original lifetime
///
/// # Note
/// Copied from `async_scoped` crate. The `async_scoped` crate cannot reuse the existing runtime.
///
unsafe fn trust_me_bro<'a, T>(
    f: impl Future<Output = T> + Send + 'a,
) -> Pin<Box<dyn Future<Output = T> + Send + 'static>> {
    unsafe {
        std::mem::transmute::<_, Pin<Box<dyn Future<Output = T> + Send>>>(
            Box::pin(f) as Pin<Box<dyn Future<Output = T>>>
        )
    }
}

#[cfg(feature = "multi-thread")]
pub(crate) fn block_on_all<I>(iter: I)
where
    I: IntoIterator,
    I::Item: Future<Output: Send + 'static> + Send,
{
    use futures::future::join_all;

    _ = TOKIO_RUNTIME
        .block_on(join_all(iter.into_iter().map(|f| async {
            tokio::spawn(unsafe { trust_me_bro(f) }).await
        })));
}

#[cfg(not(feature = "multi-thread"))]
pub(crate) fn block_on_all<I>(iter: I)
where
    I: IntoIterator,
    I::Item: Future<Output: Send>,
{
    use futures::future::join_all;
    use tokio::task::LocalSet;

    let local = LocalSet::new();
    _ = local.block_on(&TOKIO_RUNTIME, join_all(iter));
}

#[cfg(feature = "sync")]
pub trait StdError: std::error::Error + Send + Sync {}
#[cfg(feature = "sync")]
impl<T: std::error::Error + Send + Sync> StdError for T {}

#[cfg(not(feature = "sync"))]
pub trait StdError: std::error::Error {}
#[cfg(not(feature = "sync"))]
impl<T: std::error::Error> StdError for T {}
