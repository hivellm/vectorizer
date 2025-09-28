pub mod config;
pub mod methods;
pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::SummarizationConfig;
pub use manager::SummarizationManager;
pub use types::*;
pub use methods::SummarizationMethodTrait;
