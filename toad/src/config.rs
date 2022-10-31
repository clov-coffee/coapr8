#![allow(dead_code)]

use embedded_time::duration::Milliseconds;
use toad_macros::rfc_7252_doc;

use crate::retry::{Attempts, Strategy};

/// Built runtime config
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigData {
  pub(crate) token_seed: u16,
  pub(crate) con_retry_strategy: Strategy,
  pub(crate) default_leisure_millis: u64,
  pub(crate) max_retransmit_attempts: u16,
  pub(crate) nstart: u8,
  pub(crate) probing_rate_bytes_per_sec: u16,
}

impl Default for ConfigData {
  fn default() -> Self {
    Config::default().into()
  }
}

impl ConfigData {
  pub(crate) fn max_transmit_span_millis(&self) -> u64 {
    self.con_retry_strategy
        .max_time(Attempts(self.max_retransmit_attempts - 1))
        .0 as u64
  }

  pub(crate) fn max_transmit_wait_millis(&self) -> u64 {
    self.con_retry_strategy
        .max_time(Attempts(self.max_retransmit_attempts))
        .0 as u64
  }

  // TODO: adjust these on the fly based on actual timings?
  pub(crate) fn max_latency_millis(&self) -> u64 {
    100_000
  }

  pub(crate) fn expected_processing_delay_millis(&self) -> u64 {
    200
  }

  pub(crate) fn exchange_lifetime_millis(&self) -> u64 {
    self.max_transmit_span_millis()
    + (2 * self.max_latency_millis())
    + self.expected_processing_delay_millis()
  }
}

/// CoAP runtime config
///
/// Allows you to configure things like
/// "how many concurrent requests are we allowed
/// to send?" and "how long should we wait to resend
/// unacknowledged confirmable requests?"
///
/// For an example see [`Config::new`].
#[derive(Debug, Clone, Copy)]
pub struct Config {
  token_seed: Option<u16>,
  con_retry_strategy: Option<Strategy>,
  default_leisure_millis: Option<u64>,
  max_retransmit_attempts: Option<u16>,
  nstart: Option<u8>,
  probing_rate_bytes_per_sec: Option<u16>,
}

impl Default for Config {
  fn default() -> Self {
    Self { token_seed: None,
           con_retry_strategy: None,
           default_leisure_millis: None,
           max_retransmit_attempts: None,
           nstart: None,
           probing_rate_bytes_per_sec: None }
  }
}

/// Bytes / Second
#[derive(Debug, Clone, Copy)]
pub struct BytesPerSecond(pub u16);

impl Config {
  /// Creates a new (empty) runtime config
  ///
  /// ```
  /// use embedded_time::duration::Milliseconds as Millis;
  /// use toad::config::{BytesPerSecond, Config};
  /// use toad::retry::Attempts;
  /// use toad::retry::Strategy::Exponential;
  ///
  /// let config = Config::new().token_seed(35718)
  ///                           .max_concurrent_requests(142)
  ///                           .probing_rate(BytesPerSecond(10_000))
  ///                           .max_con_request_retries(Attempts(10))
  ///                           .con_retry_strategy(Exponential { init_min: Millis(500),
  ///                                                             init_max: Millis(750) });
  /// ```
  pub fn new() -> Self {
    Default::default()
  }

  /// Set the retry strategy we should use to figure out when
  /// we should resend outgoing CON requests that have not been
  /// ACKed yet.
  ///
  /// Default value:
  /// ```ignore
  /// Strategy::Exponential { init_min: Seconds(2), init_max: Seconds(3) }
  /// ```
  pub fn con_retry_strategy(mut self, strat: Strategy) -> Self {
    self.con_retry_strategy = Some(strat);
    self
  }

  /// Set the seed used to generate message [`Token`](toad_msg::Token)s.
  ///
  /// The default value is 0, although it is
  /// best practice to set this to something else.
  /// This could be a random integer, or a machine identifier.
  ///
  /// _e.g. if you're developing a swarm of
  /// smart CoAP-enabled thermostats, each one would ideally
  /// have a distinct token_seed._
  ///
  /// The purpose of the seed is to make it more
  /// difficult for an observer of unencrypted
  /// CoAP traffic to guess what the next token will be.
  ///
  /// Tokens are generated by smooshing together
  /// the 2-byte seed with an 8-byte timestamp from
  /// the system clock.
  ///
  /// ```text
  /// Core.token_seed
  /// ||
  /// xx xxxxxxxx
  ///    |      |
  ///    timestamp
  /// ```
  ///
  /// Then a hashing algorithm is used to make it opaque and
  /// reduce the size to 8 bytes.
  pub fn token_seed(mut self, token_seed: u16) -> Self {
    self.token_seed = Some(token_seed);
    self
  }

  /// Set the transmission rate that we should do our best
  /// not to exceed when waiting for:
  /// - responses to our NON requests
  /// - responses to our acked CON requests
  ///
  /// The default value is 1,000 (1KB/s)
  pub fn probing_rate(mut self, probing_rate: BytesPerSecond) -> Self {
    self.probing_rate_bytes_per_sec = Some(probing_rate.0);
    self
  }

  /// Set the number of concurrent requests we are allowed
  /// to have in-flight for each server.
  ///
  /// The default value is 1 (no concurrency)
  pub fn max_concurrent_requests(mut self, n: u8) -> Self {
    self.nstart = Some(n);
    self
  }

  /// Set the maximum number of times we should re-send
  /// confirmable requests before getting a response.
  ///
  /// The default value is 4 attempts
  pub fn max_con_request_retries(mut self, max_tries: Attempts) -> Self {
    self.max_retransmit_attempts = Some(max_tries.0);
    self
  }

  /// Set the maximum amount of time we should wait to
  /// respond to incoming multicast requests.
  ///
  /// The default value is 5 seconds.
  #[doc = rfc_7252_doc!("8.2")]
  pub fn default_leisure(mut self, default_leisure: Milliseconds<u64>) -> Self {
    self.default_leisure_millis = Some(default_leisure.0);
    self
  }
}

impl From<Config> for ConfigData {
  fn from(Config { token_seed,
                   default_leisure_millis,
                   max_retransmit_attempts,
                   nstart,
                   probing_rate_bytes_per_sec,
                   con_retry_strategy,
                   .. }: Config)
          -> Self {
    ConfigData { token_seed: token_seed.unwrap_or(0),
                 default_leisure_millis: default_leisure_millis.unwrap_or(5_000),
                 max_retransmit_attempts: max_retransmit_attempts.unwrap_or(4),
                 nstart: nstart.unwrap_or(1),
                 probing_rate_bytes_per_sec: probing_rate_bytes_per_sec.unwrap_or(1_000),
                 con_retry_strategy:
                   con_retry_strategy.unwrap_or(Strategy::Exponential { init_min:
                                                                          Milliseconds(2_000),
                                                                        init_max:
                                                                          Milliseconds(3_000) }) }
  }
}
