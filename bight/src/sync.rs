#[cfg(feature = "multi-thread")]
pub type Rc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "multi-thread"))]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(feature = "multi-thread")]
pub fn new_runtime_and_block<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(future)
}

#[cfg(not(feature = "multi-thread"))]
pub fn new_runtime_and_block<F: Future>(future: F) -> F::Output {
    use tokio::task::LocalSet;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local = LocalSet::new();
    local.block_on(&rt, future)
}

#[cfg(feature = "multi-thread")]
pub trait StdError: std::error::Error + Send + Sync {}
#[cfg(feature = "multi-thread")]
impl<T: std::error::Error + Send + Sync> StdError for T {}

#[cfg(not(feature = "multi-thread"))]
pub trait StdError: std::error::Error {}
#[cfg(not(feature = "multi-thread"))]
impl<T: std::error::Error> StdError for T {}
