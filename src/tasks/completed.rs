use color_eyre::{Result, eyre::WrapErr};
use owo_colors::OwoColorize;

use crate::{
    api::rest::{Gateway, Project, Section},
    config::Config,
    interactive,
};

use super::list::GroupBy;

#[derive(clap::Parser, Debug)]
pub struct Params {
    /// Start date (YYYY-MM-DD or ISO 8601 datetime)
    #[arg(long = "since")]
    since: Option<String>,

    /// End date (YYYY-MM-DD or ISO 8601 datetime)
    #[arg(long = "until")]
    until: Option<String>,

    /// Show tasks completed today
    #[arg(long = "today", conflicts_with_all = ["since", "until", "yesterday", "this_week", "last_week", "this_month"])]
    today: bool,

    /// Show tasks completed yesterday
    #[arg(long = "yesterday", conflicts_with_all = ["since", "until", "today", "this_week", "last_week", "this_month"])]
    yesterday: bool,

    /// Show tasks completed this week (Monday to today)
    #[arg(long = "this-week", conflicts_with_all = ["since", "until", "today", "yesterday", "last_week", "this_month"])]
    this_week: bool,

    /// Show tasks completed last week (Monday to Sunday)
    #[arg(long = "last-week", conflicts_with_all = ["since", "until", "today", "yesterday", "this_week", "this_month"])]
    last_week: bool,

    /// Show tasks completed this month (1st to today)
    #[arg(long = "this-month", conflicts_with_all = ["since", "until", "today", "yesterday", "this_week", "last_week"])]
    this_month: bool,

    /// Filter by project
    #[clap(flatten)]
    project: interactive::Selection<Project>,

    /// Filter by section
    #[clap(flatten)]
    section: interactive::Selection<Section>,

    /// Filter query (e.g., "#inbox", "today")
    #[arg(long = "filter")]
    filter: Option<String>,

    /// Limit results per page (max: 200)
    #[arg(long = "limit", default_value = "50")]
    limit: u32,

    /// Fetch all pages automatically
    #[arg(long = "all")]
    fetch_all: bool,

    /// Group tasks by specific criteria
    #[arg(long = "group-by", value_enum)]
    group_by: Option<GroupBy>,

    /// Use due date instead of completion date for filtering (supports up to 6 weeks instead of 3 months)
    #[arg(long = "by-due-date")]
    by_due_date: bool,

    /// Show task IDs in the output.
    #[arg(long = "show-id")]
    show_id: bool,
}

/// Lists completed tasks by completion date (default, up to 3 months) or due date (--by-due-date, up to 6 weeks).
///
/// # Examples
///
/// ```bash
/// # Get tasks completed today (default, by completion date)
/// doist completed
/// doist completed --today
///
/// # Get tasks completed yesterday
/// doist completed --yesterday
///
/// # Get tasks completed this week
/// doist completed --this-week
///
/// # Get tasks with specific date range
/// doist completed --since 2025-10-06 --until 2025-10-06 --by-due-date
///
/// # Get all completed tasks in October with grouping
/// doist completed --since 2025-10-01 --until 2025-10-31 --all --group-by project
/// ```
pub async fn completed(params: Params, gw: &Gateway, cfg: &Config) -> Result<()> {
    // Calculate date range based on convenience flags or use provided dates
    let (since, until) = calculate_date_range(&params)?;

    // Validate date range
    let max_weeks = if params.by_due_date { 6 } else { 12 }; // 6 weeks vs 3 months
    validate_date_range(&since, &until, max_weeks)?;

    // Fetch projects and sections for filtering
    let projects = gw.projects().await?;
    let sections = gw.sections().await?;

    let project_id = params.project.optional(&projects)?.map(|p| p.id.clone());
    let section_id = params.section.optional(&sections)?.map(|s| s.id.clone());

    let mut all_tasks = Vec::new();
    let mut cursor: Option<String> = None;
    let mut page_count = 0;

    loop {
        let response = if params.by_due_date {
            gw.completed_tasks_by_due_date(
                &since,
                &until,
                project_id.as_deref(),
                section_id.as_deref(),
                params.filter.as_deref(),
                cursor.as_deref(),
                Some(params.limit),
            )
            .await
            .wrap_err("failed to fetch completed tasks by due date")?
        } else {
            gw.completed_tasks_by_completion_date(
                &since,
                &until,
                None, // workspace_id
                project_id.as_deref(),
                section_id.as_deref(),
                None, // parent_id
                params.filter.as_deref(),
                cursor.as_deref(),
                Some(params.limit),
            )
            .await
            .wrap_err("failed to fetch completed tasks by completion date")?
        };

        let tasks_count = response.items.len();
        all_tasks.extend(response.items);
        page_count += 1;

        cursor = response.next_cursor;

        // If not fetching all or no more pages, break
        if !params.fetch_all || cursor.is_none() {
            if cursor.is_some() && !params.fetch_all {
                println!(
                    "\n{} Showing page {page_count} ({tasks_count} tasks). Use --all to fetch all pages.",
                    "ℹ".blue()
                );
            }
            break;
        }
    }

    if all_tasks.is_empty() {
        println!("No completed tasks found in the specified date range.");
        return Ok(());
    }

    // Display tasks
    display_completed_tasks(&all_tasks, &params.group_by, params.show_id, gw, cfg).await?;

    println!(
        "\n{} Total: {} completed tasks",
        "✓".green(),
        all_tasks.len()
    );

    Ok(())
}

/// Calculates the date range based on convenience flags or uses provided dates.
/// If no flags or dates are provided, defaults to today.
fn calculate_date_range(params: &Params) -> Result<(String, String)> {
    use chrono::{Datelike, Duration, Local, NaiveDate};

    let now = Local::now();
    let today = now.date_naive();

    if params.today {
        // Today: 00:00:00 to 23:59:59 in ISO 8601
        Ok((
            format!("{}T00:00:00Z", today.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", today.format("%Y-%m-%d")),
        ))
    } else if params.yesterday {
        // Yesterday: 00:00:00 to 23:59:59 in ISO 8601
        let yesterday = today - Duration::days(1);
        Ok((
            format!("{}T00:00:00Z", yesterday.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", yesterday.format("%Y-%m-%d")),
        ))
    } else if params.this_week {
        // This week: Monday 00:00:00 to today 23:59:59
        let days_from_monday = today.weekday().num_days_from_monday() as i64;
        let monday = today - Duration::days(days_from_monday);
        Ok((
            format!("{}T00:00:00Z", monday.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", today.format("%Y-%m-%d")),
        ))
    } else if params.last_week {
        // Last week: Monday to Sunday
        let days_from_monday = today.weekday().num_days_from_monday() as i64;
        let last_sunday = today - Duration::days(days_from_monday + 1);
        let last_monday = last_sunday - Duration::days(6);
        Ok((
            format!("{}T00:00:00Z", last_monday.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", last_sunday.format("%Y-%m-%d")),
        ))
    } else if params.this_month {
        // This month: 1st to today
        let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to calculate first day of month"))?;
        Ok((
            format!("{}T00:00:00Z", first_of_month.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", today.format("%Y-%m-%d")),
        ))
    } else if let (Some(since), Some(until)) = (&params.since, &params.until) {
        // Use provided dates
        Ok((since.clone(), until.clone()))
    } else {
        // Default: today
        Ok((
            format!("{}T00:00:00Z", today.format("%Y-%m-%d")),
            format!("{}T23:59:59Z", today.format("%Y-%m-%d")),
        ))
    }
}

/// Validates that the date range is within the specified maximum weeks.
fn validate_date_range(since: &str, until: &str, max_weeks: i64) -> Result<()> {
    use chrono::NaiveDate;

    let parse_date = |s: &str| -> Result<NaiveDate> {
        // Try YYYY-MM-DD format first
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Ok(date);
        }
        // Try ISO 8601 with time
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Ok(dt.date_naive());
        }
        Err(color_eyre::eyre::eyre!(
            "Invalid date format: '{}'. Use YYYY-MM-DD or ISO 8601",
            s
        ))
    };

    let since_date = parse_date(since)?;
    let until_date = parse_date(until)?;

    if until_date < since_date {
        return Err(color_eyre::eyre::eyre!(
            "'until' date must be after 'since' date"
        ));
    }

    let duration = until_date.signed_duration_since(since_date);
    if duration.num_weeks() > max_weeks {
        let time_desc = if max_weeks == 6 {
            "6 weeks"
        } else {
            "3 months"
        };
        return Err(color_eyre::eyre::eyre!(
            "Date range exceeds {} maximum (API limitation)",
            time_desc
        ));
    }

    Ok(())
}

/// Displays completed tasks with optional grouping.
async fn display_completed_tasks(
    tasks: &[crate::api::rest::Task],
    group_by: &Option<GroupBy>,
    show_id: bool,
    gw: &Gateway,
    cfg: &Config,
) -> Result<()> {
    use crate::api::tree::Tree;
    use crate::tasks::state::State;

    // Convert tasks to Tree structure for display
    let tasks_tree: Vec<Tree<crate::api::rest::Task>> =
        Tree::from_items(tasks.to_vec()).wrap_err("failed to build task tree")?;

    // Fetch related data for display
    let (projects, sections, labels) = tokio::try_join!(gw.projects(), gw.sections(), gw.labels())?;

    let state = State {
        tasks: tasks_tree,
        projects: projects.into_iter().map(|p| (p.id.clone(), p)).collect(),
        sections: sections.into_iter().map(|s| (s.id.clone(), s)).collect(),
        labels: labels.into_iter().map(|l| (l.name.clone(), l)).collect(),
        config: cfg,
    };

    // Display with grouping if specified
    if let Some(GroupBy::Project) = group_by {
        super::list::list_tasks_grouped_by_project(&state.tasks, &state, None, show_id);
    } else {
        super::list::list_tasks_with_sort(&state.tasks, &state, None, show_id);
    }

    Ok(())
}
