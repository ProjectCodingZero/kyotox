use rayon;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Failed to initialize thread pool: {0}")]
    InitError(#[from] rayon::ThreadPoolBuildError),
}

/// Calculates the recommended number of threads to use in the global thread pool.
#[inline]
fn threads_available() -> usize {
    let threads = rayon::current_num_threads();
    if threads == 1 { threads } else { threads - 1 }
}
/// Initializes the global [`rayon::ThreadPool`] thread pool with the calculated thread count.
pub fn set_thread_pool() -> Result<(), ThreadError> {
    let threads = threads_available();
    match rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(ThreadError::from(e))
        }
    }
}