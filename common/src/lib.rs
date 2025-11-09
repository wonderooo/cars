#[cfg(feature = "bucket")]
pub mod bucket;
#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "io")]
pub mod io;
#[cfg(feature = "kafka")]
pub mod kafka;
#[cfg(feature = "logging")]
pub mod logging;
#[cfg(feature = "memprof")]
pub mod memprof;
#[cfg(feature = "persistence")]
pub mod persistence;

pub async fn retry<F, O, E>(timeout: std::time::Duration, tries: usize, func: F) -> Result<O, E>
where
    F: Fn() -> Result<O, E>,
{
    let mut attempt = 0;
    loop {
        match func() {
            Ok(o) => return Ok(o),
            Err(e) => {
                attempt += 1;
                if attempt >= tries {
                    return Err(e);
                }
                tokio::time::sleep(timeout).await;
            }
        };
    }
}

pub async fn retry_async<F, Fut, O, E>(
    timeout: std::time::Duration,
    tries: usize,
    func: F,
) -> Result<O, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<O, E>>,
{
    let mut attempt = 0;
    loop {
        match func().await {
            Ok(o) => return Ok(o),
            Err(e) => {
                attempt += 1;
                if attempt >= tries {
                    return Err(e);
                }
                tokio::time::sleep(timeout).await;
            }
        };
    }
}

pub fn count_some_none<I, F, FT>(iter: I, mut field: F) -> (usize, usize)
where
    I: IntoIterator,
    F: FnMut(&I::Item) -> Option<FT>,
{
    iter.into_iter().fold((0, 0), |acc, x| match field(&x) {
        Some(_) => (acc.0 + 1, acc.1),
        None => (acc.0, acc.1 + 1),
    })
}
