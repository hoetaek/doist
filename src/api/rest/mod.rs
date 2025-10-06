//! Contains low-level structs as used by the Todoist REST API and provides some tools to work
//! with them.
//!
//! This maps parts of the [API Documentation](https://developer.todoist.com/api/v1/#overview) to
//! code that can be consumed by clients, including the actual network calls and
//! serialization/deserialization..
//!
//! To get started, take a look at [`Gateway`].
mod comment;
mod display;
mod gateway;
mod label;
mod project;
mod section;
mod task;

use serde::{Deserialize, Serialize};

pub use comment::*;
pub use display::*;
pub use gateway::*;
pub use label::*;
pub use project::*;
pub use section::*;
pub use task::*;

/// Paginated response wrapper for API v1 endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The list of results.
    pub results: Vec<T>,
    /// Cursor for fetching the next page (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
