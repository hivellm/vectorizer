pub mod config;
pub mod manager;
pub mod methods;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::SummarizationConfig;
pub use manager::SummarizationManager;
pub use methods::SummarizationMethodTrait;
pub use types::*;
