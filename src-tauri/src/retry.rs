use std::time::Duration;

/// Exponential backoff with full jitter
pub async fn retry_with_backoff<F, Fut, T, E>(
    max_attempts: u32,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    let base_delay = Duration::from_millis(500);
    let max_delay = Duration::from_secs(30);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= max_attempts {
                    log::error!("All {} retry attempts exhausted: {:?}", max_attempts, e);
                    return Err(e);
                }

                let backoff = std::cmp::min(
                    max_delay,
                    base_delay * 2u32.pow(attempt),
                );

                // Full jitter: random delay between 0 and backoff
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let jitter = rng.gen_range(Duration::from_millis(0)..backoff);

                log::warn!(
                    "Attempt {}/{} failed, retrying in {:?}: {:?}",
                    attempt + 1,
                    max_attempts,
                    jitter,
                    e
                );

                tokio::time::sleep(jitter).await;
            }
        }
    }
}
