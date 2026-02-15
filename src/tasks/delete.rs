use color_eyre::{Result, eyre::WrapErr};
use owo_colors::{OwoColorize, Stream};

use crate::{api::rest::Gateway, config::Config};

use super::filter;

#[derive(clap::Parser, Debug)]
pub struct Params {
    #[clap(flatten)]
    pub task: filter::TaskOrInteractive,
    /// Force deletion without confirmation prompt.
    #[arg(long = "force")]
    pub force: bool,
}

pub async fn delete(params: Params, gw: &Gateway, cfg: &Config) -> Result<()> {
    let (id, state) = params
        .task
        .task(gw, cfg)
        .await
        .wrap_err("no task selected for deletion")?;

    let task_name = state
        .task(&id)
        .map(|t| t.content.as_str())
        .unwrap_or("unknown");

    if !params.force {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Delete task '{task_name}'?"))
            .default(false)
            .interact()
            .wrap_err("unable to get confirmation")?;
        if !confirmed {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    gw.delete_task(&id).await?;
    println!(
        "deleted task {}",
        id.if_supports_color(Stream::Stdout, |text| text.bright_red())
    );
    Ok(())
}
