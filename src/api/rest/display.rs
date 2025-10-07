use crate::{api::tree::Tree, config::Config};

use super::{Comment, DueDateFormatter, Label, Project, Section, Task};
use chrono::Utc;
use owo_colors::{OwoColorize, Stream};

/// FullComment allows to display full comment metadata when [std::fmt::Display]ing it.
pub struct FullComment<'a>(pub &'a Comment);

impl std::fmt::Display for FullComment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FullComment(comment) = self;
        writeln!(
            f,
            "ID: {}",
            comment
                .id
                .if_supports_color(Stream::Stdout, |text| text.bright_yellow())
        )?;
        writeln!(f, "Posted: {}", comment.posted_at)?;
        writeln!(
            f,
            "Attachment: {}",
            if comment.file_attachment.is_some() {
                "Yes"
            } else {
                "No"
            }
        )?;
        write!(f, "Content: {}", comment.content)?;
        Ok(())
    }
}

/// FullLabel shows label including ID
pub struct FullLabel<'a>(pub &'a Label);

impl std::fmt::Display for FullLabel<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.0
                .id
                .if_supports_color(Stream::Stdout, |text| text.bright_yellow()),
            self.0
        )
    }
}

/// Used to display full information about a Task.
pub struct FullTask<'a>(
    pub &'a Task,
    pub Option<&'a Project>,
    pub Option<&'a Section>,
    pub Vec<&'a Label>,
    pub &'a Config,
);

impl std::fmt::Display for FullTask<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FullTask::<'_>(task, project, section, labels, config) = self;
        write!(
            f,
            "ID: {}\nPriority: {}\nContent: {}\nDescription: {}",
            task.id
                .if_supports_color(Stream::Stdout, |text| text.bright_yellow()),
            task.priority,
            task.content,
            task.description,
        )?;
        if let Some(due) = &task.due {
            write!(
                f,
                "\nDue: {}",
                DueDateFormatter(due, &config.override_time.unwrap_or_else(Utc::now))
            )?;
        }
        if !labels.is_empty() {
            write!(
                f,
                "\nLabels: {}",
                labels
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if let Some(project) = &project {
            write!(f, "\nProject: {project}")?;
        }
        if let Some(section) = &section {
            write!(f, "\nSection: {section}")?;
        }
        if let Some(deadline) = &task.deadline
            && let Some(date) = deadline.date()
        {
            write!(f, "\nDeadline: {}", date)?;
        }
        if let Some(duration) = &task.duration
            && let (Some(amount), Some(unit)) = (duration.amount(), duration.unit())
        {
            write!(f, "\nDuration: {} {}", amount, unit)?;
        }
        write!(f, "\nComments: {}", task.comment_count)?;
        Ok(())
    }
}

/// Used to display task as an item in a list.
pub struct TableTask<'a>(
    pub &'a Tree<Task>,
    pub Option<&'a Project>,
    pub Option<&'a Section>,
    pub Vec<&'a Label>,
    pub &'a Config,
    pub bool, // show_id
);

impl TableTask<'_> {
    /// Initializes a TableTask item that only displays data that is directly available from a
    /// [`Task`].
    pub fn from_task<'a>(task: &'a Tree<Task>, config: &'a Config) -> TableTask<'a> {
        TableTask(task, None, None, vec![], config, false)
    }
}

impl std::fmt::Display for TableTask<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TableTask::<'_>(task, project, section, labels, config, show_id) = self;
        let subtask_padding = if task.depth > 0 {
            format!("{}⌞ ", "  ".repeat(task.depth))
        } else {
            "".to_string()
        };

        if *show_id {
            write!(
                f,
                "{}{} {} {}",
                subtask_padding,
                task.id
                    .if_supports_color(Stream::Stdout, |text| text.bright_yellow()),
                task.priority,
                task.content,
            )?;
        } else {
            write!(f, "{}{} {}", subtask_padding, task.priority, task.content,)?;
        }

        // Show task age (days since created)
        let now = config.override_time.unwrap_or_else(Utc::now);
        let days_ago = (now - task.created_at).num_days();
        if days_ago >= 7 {
            write!(
                f,
                " {}({}일 전)",
                "".if_supports_color(Stream::Stdout, |_| "📅"),
                days_ago
            )?;
        }
        if let Some(due) = &task.due {
            write!(
                f,
                " {}",
                DueDateFormatter(due, &config.override_time.unwrap_or_else(Utc::now))
            )?;
        }
        if !labels.is_empty() {
            write!(
                f,
                " {}",
                labels
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            )?;
        }
        if let Some(deadline) = &task.deadline
            && let Some(date) = deadline.date()
        {
            write!(
                f,
                " {}⏰{}",
                "".if_supports_color(Stream::Stdout, |_| "📅"),
                date.format("%m/%d")
            )?;
        }
        if let Some(duration) = &task.duration
            && let (Some(amount), Some(unit)) = (duration.amount(), duration.unit())
        {
            let unit_symbol = match unit {
                crate::api::rest::task::DurationUnit::Minute => "⏱️",
                crate::api::rest::task::DurationUnit::Day => "📅",
            };
            write!(
                f,
                " {}{}{}",
                unit_symbol.if_supports_color(Stream::Stdout, |_| "⏱️"),
                amount,
                match unit {
                    crate::api::rest::task::DurationUnit::Minute => "m",
                    crate::api::rest::task::DurationUnit::Day => "d",
                }
            )?;
        }
        if let Some(p) = &project {
            write!(f, " [{}", p.name)?;
            if let Some(s) = &section {
                write!(f, "/{}", s.name)?;
            }
            write!(f, "]")?;
        }
        if let Some(completed_at) = &task.completed_at
            && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(completed_at)
        {
            let local_dt = dt.with_timezone(&chrono::Local);
            let formatted = local_dt.format("%m/%d %H:%M");
            write!(
                f,
                " {}{}",
                "".if_supports_color(Stream::Stdout, |_| "✅ "),
                formatted
            )?;
        }
        Ok(())
    }
}
