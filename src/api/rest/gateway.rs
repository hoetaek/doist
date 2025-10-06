use std::time::Duration;

use chrono::Utc;
use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
};
use lazy_static::lazy_static;
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use super::{
    Comment, CompletedTasksResponse, CreateComment, CreateLabel, CreateProject, CreateSection,
    CreateTask, Label, LabelID, PaginatedResponse, Project, ProjectID, Section, SectionID, Task,
    TaskDue, TaskID, UpdateTask,
};

/// Makes network calls to the Todoist API and returns structs that can then be worked with.
pub struct Gateway {
    client: ClientWithMiddleware,
    token: String,
    url: url::Url,
}

lazy_static! {
    /// The default URL that specifies the endpont to use for the Todoist API.
    pub static ref TODOIST_API_URL: url::Url = {
        url::Url::parse("https://api.todoist.com/").unwrap()
    };
}

impl Gateway {
    /// Create a new [`Gateway`].
    ///
    /// * `token` - the API token used for network calls.
    /// * `url` - the base URL to call. See [`struct@TODOIST_API_URL`]
    pub fn new(token: &str, url: &url::Url) -> Gateway {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        Gateway {
            client,
            token: token.to_string(),
            url: url.clone(),
        }
    }

    /// Retuns a [`Task`].
    ///
    /// * `id` - the ID as used by the Todoist API.
    pub async fn task(&self, id: &TaskID) -> Result<Task> {
        self.get::<(), _>(&format!("api/v1/tasks/{id}"), None)
            .await
            .wrap_err("unable to get task")
    }

    /// Returns a list of tasks as given by the API.
    ///
    /// * `filter` - a filter query as described in the [documentation](https://todoist.com/help/articles/205248842).
    pub async fn tasks(&self, filter: Option<&str>) -> Result<Vec<Task>> {
        let response: PaginatedResponse<Task> = if let Some(filter_str) = filter {
            // API v1 uses /api/v1/tasks/filter with query parameter
            self.get("api/v1/tasks/filter", Some(&[("query", filter_str)]))
                .await
                .wrap_err("unable to get tasks with filter")?
        } else {
            // Without filter, use regular /api/v1/tasks endpoint
            self.get::<(), _>("api/v1/tasks", None)
                .await
                .wrap_err("unable to get tasks")?
        };
        Ok(response.results)
    }

    /// Returns a list of completed tasks by due date range (up to 6 weeks).
    ///
    /// * `since` - Start date (ISO 8601 datetime or YYYY-MM-DD)
    /// * `until` - End date (ISO 8601 datetime or YYYY-MM-DD)
    /// * `project_id` - Optional project filter
    /// * `section_id` - Optional section filter
    /// * `filter_query` - Optional filter query
    /// * `cursor` - Pagination cursor
    /// * `limit` - Results per page (default: 50, max: 200)
    pub async fn completed_tasks_by_due_date(
        &self,
        since: &str,
        until: &str,
        project_id: Option<&str>,
        section_id: Option<&str>,
        filter_query: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<CompletedTasksResponse> {
        let mut params: Vec<(&str, &str)> = vec![("since", since), ("until", until)];

        if let Some(pid) = project_id {
            params.push(("project_id", pid));
        }
        if let Some(sid) = section_id {
            params.push(("section_id", sid));
        }
        if let Some(fq) = filter_query {
            params.push(("filter_query", fq));
        }
        if let Some(c) = cursor {
            params.push(("cursor", c));
        }

        let limit_str;
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }

        self.get("api/v1/tasks/completed/by_due_date", Some(&params))
            .await
            .wrap_err("unable to get completed tasks")
    }

    /// Returns a list of completed tasks by completion date range (up to 3 months).
    ///
    /// * `since` - Start date (ISO 8601 datetime or YYYY-MM-DD)
    /// * `until` - End date (ISO 8601 datetime or YYYY-MM-DD)
    /// * `workspace_id` - Optional workspace filter
    /// * `project_id` - Optional project filter
    /// * `section_id` - Optional section filter
    /// * `parent_id` - Optional parent task filter
    /// * `filter_query` - Optional filter query
    /// * `cursor` - Pagination cursor
    /// * `limit` - Results per page (default: 50, max: 200)
    pub async fn completed_tasks_by_completion_date(
        &self,
        since: &str,
        until: &str,
        workspace_id: Option<&str>,
        project_id: Option<&str>,
        section_id: Option<&str>,
        parent_id: Option<&str>,
        filter_query: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<CompletedTasksResponse> {
        let mut params: Vec<(&str, &str)> = vec![("since", since), ("until", until)];

        if let Some(wid) = workspace_id {
            params.push(("workspace_id", wid));
        }
        if let Some(pid) = project_id {
            params.push(("project_id", pid));
        }
        if let Some(sid) = section_id {
            params.push(("section_id", sid));
        }
        if let Some(parent) = parent_id {
            params.push(("parent_id", parent));
        }
        if let Some(fq) = filter_query {
            params.push(("filter_query", fq));
        }
        if let Some(c) = cursor {
            params.push(("cursor", c));
        }

        let limit_str;
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }

        self.get("api/v1/tasks/completed/by_completion_date", Some(&params))
            .await
            .wrap_err("unable to get completed tasks by completion date")
    }

    /// Closes a task.
    ///
    /// Equivalent to pushing the circle in the UI.
    pub async fn close(&self, id: &TaskID) -> Result<()> {
        self.post_empty(&format!("api/v1/tasks/{id}/close"), &serde_json::Map::new())
            .await
            .wrap_err("unable to close task")?;
        Ok(())
    }

    /// Complete will complete a task by first updating the due date to today, so if it's
    /// recurring, it will stop doing that.
    /// This is a bit hacky, but the REST API does not support completely closing tasks without
    /// deleting them.
    pub async fn complete(&self, id: &TaskID) -> Result<()> {
        self.update(
            id,
            &UpdateTask {
                due: Some(TaskDue::DateTime(Utc::now())),
                ..Default::default()
            },
        )
        .await
        .wrap_err("unable to complete task")?;
        self.close(id).await.wrap_err("unable to complete task")?;
        Ok(())
    }

    /// Creates a task by calling the Todoist API.
    pub async fn create(&self, task: &CreateTask) -> Result<Task> {
        self.post("api/v1/tasks", task)
            .await
            .wrap_err("unable to create task")?
            .ok_or_else(|| eyre!("unable to create task"))
    }

    /// Updates a task with the data as specified in UpdateTask.
    pub async fn update(&self, id: &TaskID, task: &UpdateTask) -> Result<()> {
        self.post_empty(&format!("api/v1/tasks/{id}"), &task)
            .await
            .wrap_err("unable to update task")?;
        Ok(())
    }

    /// Returns the list of Projects.
    pub async fn projects(&self) -> Result<Vec<Project>> {
        let response: PaginatedResponse<Project> = self
            .get::<(), _>("api/v1/projects", None)
            .await
            .wrap_err("unable to get projects")?;
        Ok(response.results)
    }

    /// Returns the list of all Sections.
    pub async fn sections(&self) -> Result<Vec<Section>> {
        let response: PaginatedResponse<Section> = self
            .get::<(), _>("api/v1/sections", None)
            .await
            .wrap_err("unable to get sections")?;
        Ok(response.results)
    }

    /// Returns the list of all Labels.
    pub async fn labels(&self) -> Result<Vec<Label>> {
        let response: PaginatedResponse<Label> = self
            .get::<(), _>("api/v1/labels", None)
            .await
            .wrap_err("unable to get labels")?;
        Ok(response.results)
    }

    /// Returns the list of all comments attached to the given Project.
    pub async fn project_comments(&self, id: &ProjectID) -> Result<Vec<Comment>> {
        let response: PaginatedResponse<Comment> = self
            .get("api/v1/comments", Some(&[("project_id", id)]))
            .await
            .wrap_err("unable to get comments")?;
        Ok(response.results)
    }

    /// Returns the list of all comments attached to the given Task.
    pub async fn task_comments(&self, id: &TaskID) -> Result<Vec<Comment>> {
        let response: PaginatedResponse<Comment> = self
            .get("api/v1/comments", Some(&[("task_id", id)]))
            .await
            .wrap_err("unable to get comments")?;
        Ok(response.results)
    }

    /// Creates a comment by calling the API.
    pub async fn create_comment(&self, comment: &CreateComment) -> Result<Comment> {
        self.post("api/v1/comments", comment)
            .await
            .wrap_err("unable to create comment")?
            .ok_or_else(|| eyre!("unable to create comment"))
    }

    /// Returns details about a single project.
    ///
    /// * `id` - the ID as used by the Todoist API.
    pub async fn project(&self, id: &ProjectID) -> Result<Project> {
        self.get::<(), _>(&format!("api/v1/projects/{id}"), None)
            .await
            .wrap_err("unable to get project")
    }

    /// Creates a project by calling the Todoist API.
    pub async fn create_project(&self, project: &CreateProject) -> Result<Project> {
        self.post("api/v1/projects", project)
            .await
            .wrap_err("unable to create project")?
            .ok_or_else(|| eyre!("unable to create project"))
    }

    /// Deletes a project by calling the Todoist API.
    pub async fn delete_project(&self, project: &ProjectID) -> Result<()> {
        self.delete(&format!("api/v1/projects/{project}"))
            .await
            .wrap_err("unable to delete project")
    }

    /// Returns details about a single section.
    ///
    /// * `id` - the ID as used by the Todoist API.
    pub async fn section(&self, id: &SectionID) -> Result<Section> {
        self.get::<(), _>(&format!("api/v1/sections/{id}"), None)
            .await
            .wrap_err("unable to get section")
    }

    /// Creates a section by calling the Todoist API.
    pub async fn create_section(&self, section: &CreateSection) -> Result<Section> {
        self.post("api/v1/sections", section)
            .await
            .wrap_err("unable to create section")?
            .ok_or_else(|| eyre!("unable to create section"))
    }

    /// Deletes a section by calling the Todoist API.
    pub async fn delete_section(&self, section: &SectionID) -> Result<()> {
        self.delete(&format!("api/v1/sections/{section}"))
            .await
            .wrap_err("unable to delete section")
    }

    /// Returns details about a single label.
    ///
    /// * `id` - the ID as used by the Todoist API.
    pub async fn label(&self, id: &LabelID) -> Result<Label> {
        self.get::<(), _>(&format!("api/v1/labels/{id}"), None)
            .await
            .wrap_err("unable to get label")
    }

    /// Creates a label by calling the Todoist API.
    pub async fn create_label(&self, label: &CreateLabel) -> Result<Label> {
        self.post("api/v1/labels", label)
            .await
            .wrap_err("unable to create label")?
            .ok_or_else(|| eyre!("unable to create label"))
    }

    /// Deletes a label by calling the Todoist API.
    pub async fn delete_label(&self, label: &LabelID) -> Result<()> {
        self.delete(&format!("api/v1/labels/{label}"))
            .await
            .wrap_err("unable to delete label")
    }

    /// Makes a GET request to the Todoist API with an optional query.
    async fn get<'a, T: 'a + Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<T>,
    ) -> Result<R> {
        let req = self
            .client
            .get(self.url.join(path)?)
            .bearer_auth(&self.token);
        let req = if let Some(q) = query {
            req.query(&q)
        } else {
            req
        };
        handle_req(req)
            .await?
            .ok_or_else(|| eyre!("Invalid response from API"))
    }

    /// Sends a POST request to the Todoist API with the given content.
    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        content: &T,
    ) -> Result<Option<R>> {
        let uuid = Uuid::new_v4();
        handle_req(
            self.client
                .post(self.url.join(path)?)
                .bearer_auth(&self.token)
                .body(serde_json::to_string(&content)?)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header("X-Request-Id", uuid.to_string()),
        )
        .await
    }

    /// Sends a DELETE request to the Todoist API.
    async fn delete(&self, path: &str) -> Result<()> {
        handle_req::<()>(
            self.client
                .delete(self.url.join(path)?)
                .bearer_auth(&self.token),
        )
        .await?;
        Ok(())
    }

    /// Same as [`Gateway::post`], but doesn't require content to be set for the POST request.
    async fn post_empty<T: Serialize>(&self, path: &str, content: &T) -> Result<()> {
        self.post::<_, Task>(path, content).await?;
        Ok(())
    }
}

/// Does the actual call to the Todoist API and handles error handling.
async fn handle_req<R: DeserializeOwned>(req: RequestBuilder) -> Result<Option<R>> {
    // TODO: implement retries/backoffs
    let resp = req
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .wrap_err("unable to send request")?;
    let status = resp.status();
    if status == StatusCode::NO_CONTENT {
        return Ok(None);
    }
    let text = resp.text().await.wrap_err("unable to read response")?;
    if !status.is_success() {
        return Err(eyre!("Bad response from API: {} - {}", status, text));
    }
    let result = serde_json::from_str(&text).wrap_err("unable to parse API response")?;
    Ok(Some(result))
}

#[cfg(test)]
mod test {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{bearer_token, method, path, query_param},
    };

    use super::*;
    use crate::api::rest::{Task, ThreadID};
    use color_eyre::Result;

    #[tokio::test]
    async fn has_authentication() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(bearer_token("hellothere"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(create_task("123", "456", "hello")),
            )
            .mount(&mock_server)
            .await;
        let gw = gateway("hellothere", &mock_server);
        let task = gw.task(&"123".to_string()).await;
        assert!(task.is_ok());
    }

    #[tokio::test]
    async fn task() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(create_task("123", "456", "hello")),
            )
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let task = gw.task(&"123".to_string()).await.unwrap();
        mock_server.verify().await;
        assert_eq!(task.id, "123");
        assert!(gw.task(&"1234".to_string()).await.is_err());
    }

    #[tokio::test]
    async fn tasks() -> Result<()> {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![
                    create_task("123", "456", "hello there"),
                    create_task("234", "567", "general kenobi"),
                ],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let tasks = gw.tasks(None).await.unwrap();
        mock_server.verify().await;
        assert_eq!(tasks.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn close_task() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123/close"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let closed = gw.close(&"123".to_string()).await;
        assert!(closed.is_ok());
    }

    #[tokio::test]
    async fn complete_task() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123/close"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let completed = gw.complete(&"123".to_string()).await;
        mock_server.verify().await;
        assert!(completed.is_ok());
    }

    #[tokio::test]
    async fn update_task() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let completed = gw
            .update(
                &"123".to_string(),
                &UpdateTask {
                    content: Some("hello".to_string()),
                    ..Default::default()
                },
            )
            .await;
        mock_server.verify().await;
        assert!(completed.is_ok());
    }

    #[tokio::test]
    async fn creates_task() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(create_task("123", "456", "hello")),
            )
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let task = gw
            .create(&CreateTask {
                content: "hello".to_string(),
                project_id: Some("456".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(task.id, "123");
    }

    #[tokio::test]
    async fn lists_projects() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![Project::new("123", "one"), Project::new("456", "two")],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let projects = gw.projects().await.unwrap();
        mock_server.verify().await;
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn show_project() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Project::new("123", "one")))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let project = gw.project(&"123".to_string()).await.unwrap();
        mock_server.verify().await;
        assert_eq!(project.id, "123");
        assert_eq!(project.name, "one");
    }

    #[tokio::test]
    async fn lists_labels() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![Label::new("123", "one"), Label::new("456", "two")],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let labels = gw.labels().await.unwrap();
        mock_server.verify().await;
        assert_eq!(labels.len(), 2);
    }

    #[tokio::test]
    async fn show_label() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/labels/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Label::new("123", "one")))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let label = gw.label(&"123".to_string()).await.unwrap();
        mock_server.verify().await;
        assert_eq!(label.id, "123");
        assert_eq!(label.name, "one");
    }

    #[tokio::test]
    async fn lists_sections() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/sections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![
                    Section::new("123", "1", "one"),
                    Section::new("456", "2", "two"),
                ],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let sections = gw.sections().await.unwrap();
        mock_server.verify().await;
        assert_eq!(sections.len(), 2);
    }

    #[tokio::test]
    async fn show_section() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/sections/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Section::new("123", "1", "one")))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let section = gw.section(&"123".to_string()).await.unwrap();
        mock_server.verify().await;
        assert_eq!(section.id, "123");
        assert_eq!(section.name, "one");
    }

    #[tokio::test]
    async fn create_project_comment() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(create_comment(
                "1",
                ThreadID::Project {
                    project_id: "123".to_string(),
                },
                "hello",
            )))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let comment = gw
            .create_comment(&CreateComment {
                thread: ThreadID::Project {
                    project_id: "123".to_string(),
                },
                content: "hello".to_string(),
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(comment.id, "1");
        assert_eq!(comment.content, "hello");
    }

    #[tokio::test]
    async fn create_task_comment() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(create_comment(
                "1",
                ThreadID::Task {
                    task_id: "123".to_string(),
                },
                "hello",
            )))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let comment = gw
            .create_comment(&CreateComment {
                thread: ThreadID::Task {
                    task_id: "123".to_string(),
                },
                content: "hello".to_string(),
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(comment.id, "1");
        assert_eq!(comment.content, "hello");
    }

    #[tokio::test]
    async fn show_comments() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/comments"))
            .and(query_param("project_id", "123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![
                    create_comment(
                        "1",
                        ThreadID::Project {
                            project_id: "123".to_string(),
                        },
                        "hello",
                    ),
                    create_comment(
                        "1",
                        ThreadID::Project {
                            project_id: "123".to_string(),
                        },
                        "there",
                    ),
                ],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/comments"))
            .and(query_param("task_id", "456"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PaginatedResponse {
                results: vec![
                    create_comment(
                        "1",
                        ThreadID::Task {
                            task_id: "456".to_string(),
                        },
                        "no",
                    ),
                    create_comment(
                        "1",
                        ThreadID::Task {
                            task_id: "456".to_string(),
                        },
                        "way",
                    ),
                ],
                next_cursor: None,
            }))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let project_comments = gw.project_comments(&"123".to_string()).await.unwrap();
        let task_comments = gw.task_comments(&"456".to_string()).await.unwrap();
        mock_server.verify().await;
        assert_eq!(project_comments.len(), 2);
        assert_eq!(project_comments[0].content, "hello");
        assert_eq!(task_comments.len(), 2);
        assert_eq!(task_comments[0].content, "no");
    }

    #[tokio::test]
    async fn creates_label() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Label::new("123", "hello")))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let label = gw
            .create_label(&CreateLabel {
                name: "hello".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(label.id, "123");
    }

    #[tokio::test]
    async fn delete_label() {
        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/labels/123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let closed = gw.delete_label(&"123".to_string()).await;
        assert!(closed.is_ok());
    }

    #[tokio::test]
    async fn creates_project() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Project::new("123", "hello")))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let project = gw
            .create_project(&CreateProject {
                name: "hello".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(project.id, "123");
    }

    #[tokio::test]
    async fn delete_project() {
        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/projects/123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let closed = gw.delete_project(&"123".to_string()).await;
        assert!(closed.is_ok());
    }

    fn gateway(token: &str, ms: &MockServer) -> Gateway {
        Gateway::new(token, &ms.uri().parse().unwrap())
    }

    #[tokio::test]
    async fn creates_section() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/sections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(Section::new("123", "456", "heya")),
            )
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let section = gw
            .create_section(&CreateSection {
                name: "hello".to_string(),
                project_id: "456".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        mock_server.verify().await;
        assert_eq!(section.id, "123");
        assert_eq!(section.project_id, "456");
    }

    #[tokio::test]
    async fn delete_section() {
        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/sections/123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        let gw = gateway("", &mock_server);
        let closed = gw.delete_section(&"123".to_string()).await;
        assert!(closed.is_ok());
    }

    fn create_task(id: &str, project_id: &str, content: &str) -> Task {
        crate::api::rest::Task {
            project_id: project_id.to_string(),
            ..crate::api::rest::Task::new(id, content)
        }
    }

    fn create_comment(id: &str, tid: ThreadID, content: &str) -> Comment {
        Comment {
            id: id.to_string(),
            posted_uid: None,
            thread: Some(tid),
            posted_at: Utc::now(),
            content: content.to_string(),
            file_attachment: None,
            uids_to_notify: vec![],
            is_deleted: false,
            reactions: None,
        }
    }

    #[tokio::test]
    async fn completed_tasks_by_due_date() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks/completed/by_due_date"))
            .and(query_param("since", "2025-09-01"))
            .and(query_param("until", "2025-10-06"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(CompletedTasksResponse {
                    items: vec![
                        create_task("1", "1000001", "Completed task 1"),
                        create_task("2", "1000001", "Completed task 2"),
                    ],
                    next_cursor: Some("cursor123".to_string()),
                }),
            )
            .mount(&mock_server)
            .await;

        let gw = gateway("", &mock_server);
        let response = gw
            .completed_tasks_by_due_date("2025-09-01", "2025-10-06", None, None, None, None, None)
            .await
            .unwrap();

        mock_server.verify().await;
        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].content, "Completed task 1");
        assert_eq!(response.next_cursor, Some("cursor123".to_string()));
    }

    #[tokio::test]
    async fn completed_tasks_by_completion_date() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks/completed/by_completion_date"))
            .and(query_param("since", "2025-10-01"))
            .and(query_param("until", "2025-10-06"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(CompletedTasksResponse {
                    items: vec![
                        create_task("1", "1000001", "Completed today 1"),
                        create_task("2", "1000001", "Completed today 2"),
                        create_task("3", "1000002", "Completed today 3"),
                    ],
                    next_cursor: Some("cursor456".to_string()),
                }),
            )
            .mount(&mock_server)
            .await;

        let gw = gateway("", &mock_server);
        let response = gw
            .completed_tasks_by_completion_date(
                "2025-10-01",
                "2025-10-06",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        mock_server.verify().await;
        assert_eq!(response.items.len(), 3);
        assert_eq!(response.items[0].content, "Completed today 1");
        assert_eq!(response.items[2].content, "Completed today 3");
        assert_eq!(response.next_cursor, Some("cursor456".to_string()));
    }
}
