use color_eyre::Result;

use crate::{
    api::{
        self,
        rest::{DurationUnit, Gateway, Project, Section, TaskDue, UpdateTask},
    },
    config::Config,
    interactive,
    labels::{self, LabelSelect},
    tasks::{Priority, filter::TaskOrInteractive},
};

#[derive(clap::Parser, Debug)]
pub struct Params {
    #[clap(flatten)]
    pub task: TaskOrInteractive,
    /// Name of a task
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
    #[arg(short = 'd', long = "due")]
    pub due: Option<String>,
    /// Description of a task.
    #[arg(short = 'D', long = "desc")]
    pub desc: Option<String>,
    /// Sets the priority on the task. The lower the priority the more urgent the task.
    #[arg(value_enum, short = 'p', long = "priority")]
    pub priority: Option<Priority>,
    /// Set deadline with a date in YYYY-MM-DD format.
    #[arg(long = "deadline")]
    pub deadline: Option<String>,
    /// Set task duration with format "<amount>:<unit>" (e.g., "30:minute" or "2:day"). Requires --due to be specified.
    #[arg(long = "duration")]
    pub duration: Option<String>,
    #[clap(flatten)]
    pub labels: LabelSelect,
    /// Move task to a different project.
    #[clap(flatten)]
    pub project: interactive::Selection<Project>,
    /// Move task to a different section.
    #[clap(flatten)]
    pub section: interactive::Selection<Section>,
}

impl Params {
    pub fn new(id: api::rest::TaskID) -> Self {
        Self {
            task: TaskOrInteractive::with_id(id),
            name: None,
            due: None,
            desc: None,
            priority: None,
            deadline: None,
            duration: None,
            labels: LabelSelect::default(),
            project: Default::default(),
            section: Default::default(),
        }
    }
}

pub async fn edit(params: Params, gw: &Gateway, cfg: &Config) -> Result<()> {
    let labels = {
        let labels = params
            .labels
            .labels(&gw.labels().await?, labels::Selection::AllowEmpty)?;
        if labels.is_empty() {
            None
        } else {
            Some(labels.into_iter().map(|l| l.name).collect())
        }
    };
    let mut update = UpdateTask {
        content: params.name,
        description: params.desc,
        priority: params.priority.map(|p| p.into()),
        labels,
        ..Default::default()
    };
    let due_provided = params.due.is_some();
    if let Some(due) = params.due {
        update.due = Some(TaskDue::String(due))
    }
    if let Some(deadline_str) = params.deadline {
        if chrono::NaiveDate::parse_from_str(&deadline_str, "%Y-%m-%d").is_ok() {
            update.deadline_date = Some(deadline_str);
            update.deadline_lang = Some("en".to_string());
        } else {
            return Err(color_eyre::eyre::eyre!(
                "Invalid deadline format. Use YYYY-MM-DD format."
            ));
        }
    }
    if let Some(duration_str) = params.duration {
        if update.due.is_none() && !due_provided {
            return Err(color_eyre::eyre::eyre!(
                "Duration requires a due date. Use --due option when specifying duration."
            ));
        }
        if let Some((amount_str, unit_str)) = duration_str.split_once(':') {
            if let Ok(amount) = amount_str.parse::<u32>() {
                if amount == 0 {
                    return Err(color_eyre::eyre::eyre!(
                        "Duration amount must be greater than zero."
                    ));
                }
                let unit = match unit_str {
                    "minute" => DurationUnit::Minute,
                    "day" => DurationUnit::Day,
                    _ => {
                        return Err(color_eyre::eyre::eyre!(
                            "Invalid duration unit. Use 'minute' or 'day'."
                        ));
                    }
                };
                update.duration = Some(amount);
                update.duration_unit = Some(unit);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Invalid duration amount. Must be a positive integer."
                ));
            }
        } else {
            return Err(color_eyre::eyre::eyre!(
                "Invalid duration format. Use '<amount>:<unit>' format (e.g., '30:minute' or '2:day')."
            ));
        }
    }
    // Resolve project and section selections
    let projects = gw.projects().await?;
    let sections = gw.sections().await?;
    let move_project = params.project.optional(&projects)?.map(|p| p.id.clone());
    let move_section = params.section.optional(&sections)?.map(|s| s.id.clone());

    let task_id = params.task.task_id(gw, cfg).await?;
    let has_update = update.content.is_some()
        || update.description.is_some()
        || update.priority.is_some()
        || update.labels.is_some()
        || update.due.is_some()
        || update.deadline_date.is_some()
        || update.duration.is_some();
    if has_update {
        gw.update(&task_id, &update).await?;
    }

    if move_project.is_some() || move_section.is_some() {
        gw.move_task(&task_id, move_project.as_ref(), move_section.as_ref())
            .await?;
    }
    Ok(())
}
