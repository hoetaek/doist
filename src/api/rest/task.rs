use core::fmt;
use std::fmt::Display;

use crate::api::serialize::todoist_rfc3339;
use crate::api::tree::Treeable;
use chrono::{DateTime, FixedOffset, Utc};
use owo_colors::{OwoColorize, Stream};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{ProjectID, SectionID};

/// TaskID describes the unique ID of a [`Task`].
pub type TaskID = String;
/// UserID is the unique ID of a User.
pub type UserID = String;

/// Task describes a Task from the Todoist API.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#tag/Tasks).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Task {
    /// Unique ID of a Task.
    pub id: TaskID,
    /// User ID of the person who created the task.
    pub user_id: Option<UserID>,
    /// Shows which [`super::Project`] the Task belongs to.
    pub project_id: ProjectID,
    /// Set if the Task is also in a subsection of a Project.
    pub section_id: Option<SectionID>,
    /// The main content of the Task, also known as Task name.
    pub content: String,
    /// Description is the description found under the content.
    pub description: String,
    /// Completed is set if this task was completed (API v1 uses "checked").
    #[serde(alias = "checked")]
    pub is_completed: bool,
    /// All associated [`super::Label`]s to this Task. Just label names are used here.
    pub labels: Vec<String>,
    /// If set, this Task is a subtask of another.
    pub parent_id: Option<TaskID>,
    /// Order the order within the subtasks of a Task (API v1 uses "child_order").
    #[serde(alias = "child_order")]
    pub order: isize,
    /// Priority is how urgent the task is.
    pub priority: Priority,
    /// The due date of the Task.
    pub due: Option<DueDate>,
    /// Deadline for the Task.
    pub deadline: Option<Deadline>,
    /// Duration for the Task.
    pub duration: Option<Duration>,
    /// Links the Task to a URL in the Todoist UI (optional in v1).
    #[serde(default = "default_url")]
    pub url: Url,
    /// How many comments are written for this Task (API v1 uses "note_count").
    #[serde(alias = "note_count", default)]
    pub comment_count: usize,
    /// Who created this task (API v1 uses "added_by_uid").
    #[serde(alias = "added_by_uid")]
    pub creator_id: UserID,
    /// Who this task is assigned to (API v1 uses "responsible_uid").
    #[serde(alias = "responsible_uid")]
    pub assignee_id: Option<UserID>,
    /// Who assigned this task (API v1 uses "assigned_by_uid").
    #[serde(alias = "assigned_by_uid")]
    pub assigner_id: Option<UserID>,
    /// Exact date when the task was created (API v1 uses "added_at").
    #[serde(alias = "added_at", serialize_with = "todoist_rfc3339")]
    pub created_at: DateTime<Utc>,
    /// Whether the task is deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// When the task was completed (API v1 field).
    #[serde(default)]
    pub completed_at: Option<String>,
    /// When the task was last updated (API v1 field).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Day order (API v1 field).
    #[serde(default)]
    pub day_order: Option<isize>,
    /// Whether subtasks are collapsed (API v1 field).
    #[serde(default)]
    pub is_collapsed: bool,
}

fn default_url() -> Url {
    "http://localhost".parse().unwrap()
}

impl Treeable for Task {
    type ID = TaskID;

    fn id(&self) -> TaskID {
        self.id.clone()
    }

    fn parent_id(&self) -> Option<TaskID> {
        self.parent_id.clone()
    }

    fn reset_parent(&mut self) {
        self.parent_id = None;
    }
}

impl Ord for Task {
    /// Sorts on a best-attempt to make it sort similar to the Todoist UI.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Exact times ignore even priority in the UI
        match (
            self.due.as_ref().and_then(|d| d.exact_datetime()),
            other.due.as_ref().and_then(|d| d.exact_datetime()),
        ) {
            (Some(left), Some(right)) => match left.cmp(&right) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            },
            (Some(_left), None) => return std::cmp::Ordering::Less,
            (None, Some(_right)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }

        // Lower priority in API is lower in list
        match self.priority.cmp(&other.priority).reverse() {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.order.cmp(&other.order) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority as is given from the Todoist API.
///
/// 1 for Normal up to 4 for Urgent.
#[derive(
    Default, Debug, Copy, Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum Priority {
    /// p1 in the Todoist UI.
    #[default]
    Normal = 1,
    /// p3 in the Todoist UI.
    High = 2,
    /// p2 in the Todoist UI.
    VeryHigh = 3,
    /// p1 in the Todoist UI.
    Urgent = 4,
}

impl Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The priority display is reversed as in the actual desktop client compared to the API.
        match self {
            Priority::Normal => write!(f, "p4"),
            Priority::High => write!(
                f,
                "{}",
                "p3".if_supports_color(Stream::Stdout, |text| text.blue())
            ),
            Priority::VeryHigh => write!(
                f,
                "{}",
                "p2".if_supports_color(Stream::Stdout, |text| text.yellow())
            ),
            Priority::Urgent => write!(
                f,
                "{}",
                "p1".if_supports_color(Stream::Stdout, |text| text.red())
            ),
        }
    }
}

/// ExactTime exists in DueDate if this is an exact DueDate.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ExactTime {
    /// Exact DateTime for when the task is due.
    pub datetime: DateTime<FixedOffset>,
    /// Timezone string or UTC offset. // TODO: currently will not interpret correctly if it's a UTC offset.
    pub timezone: String,
}

impl Display for ExactTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(tz) = self.timezone.parse::<chrono_tz::Tz>() {
            write!(f, "{}", self.datetime.with_timezone(&tz))
        } else {
            write!(f, "{}", self.datetime)
        }
    }
}

/// DueDate is the Due object from the Todoist API.
///
/// Mostly contains human-readable content for easier display.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DueDate {
    /// Human-redable form of the due date.
    #[serde(rename = "string")]
    pub string: String,
    /// The date on which the Task is due (as string from API v1).
    /// Can be in format: YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS, or YYYY-MM-DDTHH:MM:SSZ
    pub date: String,
    /// Timezone (null for full-day or floating dates, timezone name for fixed dates).
    pub timezone: Option<String>,
    /// Language code for parsing the string.
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Lets us know if it is recurring (reopens after close).
    pub is_recurring: bool,
}

fn default_lang() -> String {
    "en".to_string()
}

impl DueDate {
    /// Get the date part as NaiveDate.
    pub fn date_naive(&self) -> Option<chrono::NaiveDate> {
        // Try parsing as just date
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&self.date, "%Y-%m-%d") {
            return Some(date);
        }
        // Try parsing with time (floating)
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%dT%H:%M:%S") {
            return Some(dt.date());
        }
        // Try parsing with time and milliseconds (floating)
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%dT%H:%M:%S.%f") {
            return Some(dt.date());
        }
        // Try parsing with timezone
        if let Ok(dt) = DateTime::parse_from_rfc3339(&self.date) {
            return Some(dt.date_naive());
        }
        None
    }

    /// Get the exact datetime if available.
    pub fn exact_datetime(&self) -> Option<DateTime<FixedOffset>> {
        // Try parsing with timezone (RFC 3339)
        if let Ok(dt) = DateTime::parse_from_rfc3339(&self.date) {
            return Some(dt);
        }
        None
    }
}

/// Formats a [`DueDate`] using the given [`DateTime`], by coloring the output based on if it's
/// too late or too soon.
pub struct DueDateFormatter<'a>(pub &'a DueDate, pub &'a DateTime<Utc>);

/// Deadline object from the Todoist API.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Deadline {
    /// Structured deadline with date and optional language.
    Structured {
        /// Date in format YYYY-MM-DD corrected to user's timezone.
        date: chrono::NaiveDate,
        /// Language to use for parsing the deadline string.
        #[serde(skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
    },
    /// Raw object from API v1 (property-based).
    Raw(serde_json::Map<String, serde_json::Value>),
}

/// Duration object from the Todoist API.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Duration {
    /// Structured duration with amount and unit.
    Structured {
        /// Amount of time the task will take (positive integer).
        amount: u32,
        /// Unit of time - either "minute" or "day".
        unit: DurationUnit,
    },
    /// Raw object from API v1 (property-based).
    Raw(serde_json::Map<String, serde_json::Value>),
}

impl Deadline {
    /// Get the date from the deadline, regardless of variant.
    pub fn date(&self) -> Option<chrono::NaiveDate> {
        match self {
            Deadline::Structured { date, .. } => Some(*date),
            Deadline::Raw(map) => map
                .get("date")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        }
    }
}

impl Duration {
    /// Get the amount from the duration, regardless of variant.
    pub fn amount(&self) -> Option<u32> {
        match self {
            Duration::Structured { amount, .. } => Some(*amount),
            Duration::Raw(map) => map
                .values()
                .find_map(|v| v.as_u64())
                .and_then(|n| u32::try_from(n).ok()),
        }
    }

    /// Get the unit from the duration, regardless of variant.
    pub fn unit(&self) -> Option<DurationUnit> {
        match self {
            Duration::Structured { unit, .. } => Some(unit.clone()),
            Duration::Raw(map) => {
                // Try to infer from property values or keys
                for (key, value) in map.iter() {
                    if key.contains("minute") || value.as_str() == Some("minute") {
                        return Some(DurationUnit::Minute);
                    } else if key.contains("day") || value.as_str() == Some("day") {
                        return Some(DurationUnit::Day);
                    }
                }
                None
            }
        }
    }
}

/// Duration unit enum.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DurationUnit {
    /// Time unit in minutes.
    Minute,
    /// Time unit in days.
    Day,
}

impl Display for DurationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationUnit::Minute => write!(f, "minute"),
            DurationUnit::Day => write!(f, "day"),
        }
    }
}

impl Display for DueDateFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_recurring {
            write!(
                f,
                "{}",
                "[REPEAT] ".if_supports_color(Stream::Stdout, |_| "🔁 ")
            )?;
        }
        if let Some(exact) = self.0.exact_datetime() {
            if exact >= *self.1 {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_green())
                )
            } else {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_red())
                )
            }
        } else if let Some(date) = self.0.date_naive() {
            if date >= self.1.date_naive() {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_green())
                )
            } else {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_red())
                )
            }
        } else {
            write!(
                f,
                "{}",
                self.0
                    .string
                    .if_supports_color(Stream::Stdout, |text| text.bright_green())
            )
        }
    }
}

/// Human representation of the due date.
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskDue {
    /// Human readable representation of the date.
    #[serde(rename = "due_string")]
    String(String),
    /// Loose target date with no exact time. TODO: should use way to encode it as a type.
    #[serde(rename = "due_date")]
    Date(String),
    /// Exact DateTime in UTC for the due date.
    #[serde(rename = "due_datetime", serialize_with = "todoist_rfc3339")]
    DateTime(DateTime<Utc>),
}
/// Command used with [`super::Gateway::create`] to create a new Task.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateTask {
    /// Sets the [`Task::content`] on the new [`Task`]. (Required)
    pub content: String,
    /// Sets the [`Task::description`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Sets the [`Task::project_id`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectID>,
    /// Sets the [`Task::section_id`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<SectionID>,
    /// Sets the [`Task::parent_id`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<TaskID>,
    /// Sets the [`Task::order`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<isize>,
    /// Sets the [`Task::labels`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// Sets the [`Task::priority`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    /// Sets the assignee_id on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<u32>,
    /// Sets the [`Task::due`] on the new [`Task`].
    #[serde(flatten)]
    pub due: Option<TaskDue>,
    /// If due is [TaskDue::String], this two-letter code optionally specifies the language if it's not english.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_lang: Option<String>,
    /// Sets the [`Task::deadline`] on the new [`Task`].
    #[serde(rename = "deadline_date", skip_serializing_if = "Option::is_none")]
    pub deadline_date: Option<String>,
    /// Sets the [`Task::duration`] on the new [`Task`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Unit of time for duration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_unit: Option<DurationUnit>,
}

/// Command used with [`super::Gateway::update`] to update a [`Task`].
///
/// Each field is optional, so if something exists, that part of the [`Task`] will get overwritten.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UpdateTask {
    /// Overwrites [`Task::content`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Overwrites [`Task::description`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Overwrites [`Task::labels`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// Overwrites [`Task::priority`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    /// Overwrites [`Task::due`] if set.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub due: Option<TaskDue>,
    /// If due is [TaskDue::String], this two-letter code optionally specifies the language if it's not english.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_lang: Option<String>,
    /// Overwrites [`Task::assignee`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<UserID>,
    /// Sets the deadline on the task.
    #[serde(rename = "deadline_date", skip_serializing_if = "Option::is_none")]
    pub deadline_date: Option<String>,
    /// Language for deadline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_lang: Option<String>,
    /// Sets the duration on the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Unit of time for duration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_unit: Option<DurationUnit>,
}

#[cfg(test)]
impl Task {
    /// This is initializer is used for tests, as in general the tool relies on the API and not
    /// local state.
    pub fn new(id: &str, content: &str) -> Task {
        Task {
            id: id.to_string(),
            user_id: None,
            project_id: "".to_string(),
            section_id: None,
            content: content.to_string(),
            description: String::new(),
            is_completed: false,
            labels: Vec::new(),
            parent_id: None,
            order: 0,
            priority: Priority::default(),
            due: None,
            deadline: None,
            duration: None,
            url: "http://localhost".to_string().parse().unwrap(),
            comment_count: 0,
            creator_id: "0".to_string(),
            assignee_id: None,
            assigner_id: None,
            created_at: Utc::now(),
            is_deleted: false,
            completed_at: None,
            updated_at: None,
            day_order: None,
            is_collapsed: false,
        }
    }
}

/// Response for completed tasks by due date endpoint.
///
/// API v1 returns `{items: [...], next_cursor: "..."}` instead of `{results: [...], next_cursor: "..."}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTasksResponse {
    /// The list of completed tasks.
    pub items: Vec<Task>,
    /// Cursor for fetching the next page (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
