use owo_colors::{OwoColorize, Stream};
use serde::{Deserialize, Serialize};

use super::ProjectID;

/// SectionID is the unique ID of a [`Section`].
pub type SectionID = String;

/// Section describes a subsection of a [`super::Project`].
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/rest/v1/#sections).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Section {
    /// The unique ID of this section.
    pub id: SectionID,
    /// Project ID that this section belongs to.
    pub project_id: ProjectID,
    /// Position of the section amonst sections from the same project (API v1 uses "section_order").
    #[serde(alias = "section_order", default)]
    pub order: isize,
    /// The actual name of the section.
    pub name: String,
    /// User ID of the person who created the section.
    #[serde(default)]
    pub user_id: Option<String>,
    /// When the section was created.
    #[serde(default)]
    pub added_at: Option<String>,
    /// When the section was last updated.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// When the section was archived.
    #[serde(default)]
    pub archived_at: Option<String>,
    /// Whether the section is archived.
    #[serde(default)]
    pub is_archived: bool,
    /// Whether the section is deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// Whether the section is collapsed.
    #[serde(default)]
    pub is_collapsed: bool,
}

impl Ord for Section {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.order.cmp(&other.order) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Section {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.id
                .if_supports_color(Stream::Stdout, |text| text.bright_yellow()),
            self.name
        )
    }
}

/// Command used with [`super::Gateway::create_section`] to create a new [`Section`].
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateSection {
    /// Name of the project to create.
    pub name: String,
    /// The project of which this section is part of
    pub project_id: ProjectID,
    /// Order of the section in lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<isize>,
}

#[cfg(test)]
impl Section {
    /// This is initializer is used for tests, as in general the tool relies on the API and not
    /// local state.
    pub fn new(id: &str, project_id: &str, name: &str) -> Section {
        Section {
            id: id.to_string(),
            project_id: project_id.to_string(),
            name: name.to_string(),
            order: 0,
            user_id: None,
            added_at: None,
            updated_at: None,
            archived_at: None,
            is_archived: false,
            is_deleted: false,
            is_collapsed: false,
        }
    }
}
