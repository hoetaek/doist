use crate::api::tree::Treeable;
use owo_colors::{OwoColorize, Stream};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// ProjectID is the unique ID of a [`Project`]
pub type ProjectID = String;
/// ProjectSyncID is an identifier to mark between copies of shared projects.
pub type ProjectSyncID = String;

/// Project as described by the Todoist API.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/rest/v1/#projects).
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Project {
    /// ID of the Project.
    pub id: ProjectID,
    /// The direct parent of the project if it exists.
    pub parent_id: Option<ProjectID>,
    /// The name of the Project. Displayed in the project list in the UI.
    pub name: String,
    /// Color as used by the Todoist UI.
    pub color: String,
    /// Whether the project is shared with someone else.
    pub is_shared: bool,
    /// Project order under the same parent (API v1 uses "child_order").
    #[serde(alias = "child_order", default)]
    pub order: isize,
    /// This marks the project as the initial Inbox project if it exists (API v1 uses "inbox_project").
    #[serde(alias = "inbox_project", default)]
    pub is_inbox_project: bool,
    /// Toggle to mark this project as a favorite.
    pub is_favorite: bool,
    /// View style to show in todoist clients.
    #[serde(default)]
    pub view_style: ViewStyle,
    /// Whether tasks can be assigned in this project.
    #[serde(default)]
    pub can_assign_tasks: bool,
    /// User ID of the person who created the project.
    #[serde(default)]
    pub creator_uid: Option<String>,
    /// When the project was created.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Whether the project is archived.
    #[serde(default)]
    pub is_archived: bool,
    /// Whether the project is deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// Whether the project is frozen.
    #[serde(default)]
    pub is_frozen: bool,
    /// When the project was last updated.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Default ordering.
    #[serde(default)]
    pub default_order: Option<isize>,
    /// Project description.
    #[serde(default)]
    pub description: Option<String>,
    /// Public key for shared projects.
    #[serde(default)]
    pub public_key: Option<String>,
    /// Whether the project is collapsed.
    #[serde(default)]
    pub is_collapsed: bool,
    /// URL to the Todoist UI (optional in v1).
    #[serde(default = "default_project_url")]
    pub url: Url,
    /// This markes the project as a TeamInbox project if it exists (removed from v1, kept for compatibility).
    #[serde(default)]
    pub is_team_inbox: bool,
    /// How many project comments (removed from v1, kept for compatibility).
    #[serde(default)]
    pub comment_count: usize,
}

fn default_project_url() -> Url {
    "http://localhost".parse().unwrap()
}

/// ViewStyle for viewing of the project in different clients.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/rest/v2/#projects).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ViewStyle {
    /// Project as list view (default).
    List,
    /// Project as board view.
    Board,
    /// Project as calendar view.
    Calendar,
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self::List
    }
}

impl Treeable for Project {
    type ID = ProjectID;

    fn id(&self) -> ProjectID {
        self.id.clone()
    }

    fn parent_id(&self) -> Option<ProjectID> {
        self.parent_id.clone()
    }

    fn reset_parent(&mut self) {
        self.parent_id = None;
    }
}

impl std::fmt::Display for Project {
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

/// Command used with [`super::Gateway::create_project`] to create a new [`Project`].
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateProject {
    /// Name of the project to create.
    pub name: String,
    /// Makes the newly created project a child of this parent project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ProjectID>,
    /// Color of the project icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Mark as favorite or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    /// Sets the view style of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_style: Option<ViewStyle>,
}

#[cfg(test)]
impl Project {
    /// This is initializer is used for tests, as in general the tool relies on the API and not
    /// local state.
    pub fn new(id: &str, name: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            parent_id: None,
            color: "".to_string(),
            is_shared: false,
            order: 0,
            is_inbox_project: false,
            is_favorite: false,
            view_style: Default::default(),
            can_assign_tasks: false,
            creator_uid: None,
            created_at: None,
            is_archived: false,
            is_deleted: false,
            is_frozen: false,
            updated_at: None,
            default_order: None,
            description: None,
            public_key: None,
            is_collapsed: false,
            url: "http://localhost".to_string().parse().unwrap(),
            is_team_inbox: false,
            comment_count: 0,
        }
    }
}
