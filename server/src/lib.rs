#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate rand;

#[macro_use]
extern crate log;

extern crate time;
extern crate chrono;
extern crate timer;

extern crate opcua_core;

type DateTimeUTC = chrono::DateTime<chrono::UTC>;

mod services;
mod comms;
mod session;

pub mod server;

pub mod subscriptions;

pub mod config;

pub mod address_space;

pub mod prelude {
    pub use opcua_core::prelude::*;
    pub use address_space::*;
    pub use config::*;
    pub use server::*;
    pub use subscriptions::*;
}

/// Constants that govern the internal workings of the server impl.
mod constants {
    use opcua_core::types::Double;

    /// Minimum sampling interval in seconds allowed by clients on subscriptions or monitored_items
    pub const MIN_SAMPLING_INTERVAL: Double = 0.05f64;
    /// Maximum data change queue allowed by clients on monitored items
    pub const MAX_DATA_CHANGE_QUEUE_SIZE: usize = 10;
    /// The default size of preallocated vecs of monitored items per subscription
    pub const DEFAULT_MONITORED_ITEM_CAPACITY: usize = 100;
    // Sampling interval in MS used internally to poll subscriptions. The more finegrained this is
    // the more often subscriptions will be checked to see if their subscription interval has elapsed
    // therefore the value should be < min sampling interval
    pub const SUBSCRIPTION_TIMER_RATE_MS: i64 = 50;
}

#[cfg(test)]
mod tests;
