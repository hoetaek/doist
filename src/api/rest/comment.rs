use serde::{Deserialize, Serialize, Deserializer};

use crate::api::serialize::todoist_rfc3339;

use super::{ProjectID, TaskID};

/// Deserialize null as empty vec
fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// CommentID describes the unique ID of a [`Comment`].
pub type CommentID = String;

/// ThreadID is the ID of the location where the comment is posted.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ThreadID {
    /// The ID of the project this comment is attached to.
    Project {
        /// The ID of the [`super::Project`].
        project_id: ProjectID,
    },
    /// The ID of the task this comment is attached to.
    Task {
        /// The ID of the [`super::Task`].
        task_id: TaskID,
    },
}

/// Comment describes a Comment from the Todoist API.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#tag/Comments)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    /// The unique ID of a comment.
    pub id: CommentID,
    /// User ID who posted the comment
    #[serde(default)]
    pub posted_uid: Option<String>,
    /// Where the comment is attached to (task_id or project_id).
    /// May be None in API v1 responses where the thread context is implied.
    #[serde(flatten, skip_serializing_if = "Option::is_none", default)]
    pub thread: Option<ThreadID>,
    /// The date when the comment was posted.
    #[serde(serialize_with = "todoist_rfc3339")]
    pub posted_at: chrono::DateTime<chrono::Utc>,
    /// Contains the comment text with markdown.
    pub content: String,
    /// Optional attachment file description.
    #[serde(alias = "attachment", default)]
    pub file_attachment: Option<Attachment>,
    /// User IDs to notify
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub uids_to_notify: Vec<String>,
    /// Whether the comment is deleted
    #[serde(default)]
    pub is_deleted: bool,
    /// Reactions to the comment
    #[serde(default)]
    pub reactions: Option<serde_json::Map<String, serde_json::Value>>,
}

/// An optional attachment file attached to a comment.
/// TODO: empty for now, so it acts as a marker.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {}

/// CreateComment allows to create a new comment through the API.
#[derive(Debug, Serialize)]
pub struct CreateComment {
    /// The thread to attach the comment to.
    #[serde(flatten)]
    pub thread: ThreadID,
    /// The text of the comment. Supports markdown.
    pub content: String,
    // TODO: pub attachment: Option<Attachment>,
}
