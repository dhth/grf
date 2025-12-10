use crate::cli::{Args, GraphQCommand};
use crate::cmds::{ConsoleError, QueryBehaviour, handle_console_cmd, handle_query_cmd};
use crate::view::ConsoleConfig;
use clap::Parser;
use etcetera::{BaseStrategy, HomeDirError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("couldn't determine your home directory: {0}")]
    XdgError(#[from] HomeDirError),
    #[error(transparent)]
    ConsoleCmdError(#[from] ConsoleError),
    #[error("0")]
    InvalidCLIUsage(&'static str),
    #[error(transparent)]
    Uncategorised(#[from] anyhow::Error),
}

impl AppError {
    pub fn follow_up(&self) -> Option<String> {
        match self {
            AppError::XdgError(_) => None,
            AppError::ConsoleCmdError(_) => None,
            AppError::InvalidCLIUsage(_) => None,
            AppError::Uncategorised(_) => None,
        }
    }

    pub fn is_unexpected(&self) -> bool {
        match self {
            AppError::XdgError(_) => true,
            AppError::ConsoleCmdError(_) => false,
            AppError::InvalidCLIUsage(_) => false,
            AppError::Uncategorised(_) => false,
        }
    }
}

pub async fn run() -> Result<(), AppError> {
    let xdg = etcetera::choose_base_strategy()?;
    crate::logging::setup(&xdg)?;
    let args = Args::parse();

    if args.debug {
        print!("DEBUG INFO\n{args}");
        return Ok(());
    }

    match args.command {
        GraphQCommand::Console {
            page_results,
            write_results,
            results_directory,
            results_format,
        } => {
            let console_config = ConsoleConfig {
                page_results,
                write_results,
                results_directory,
                results_format,
                history_file_path: xdg.data_dir().join("grf").join("history.txt"),
            };

            handle_console_cmd(console_config).await?;
        }
        GraphQCommand::Query {
            query,
            page_results,
            benchmark,
            bench_num_runs,
            bench_num_warmup_runs,
            print_query,
            write_results,
            results_directory,
            results_format,
        } => {
            if benchmark && write_results {
                return Err(AppError::InvalidCLIUsage(
                    "cannot benchmark and write results at the same time",
                ));
            }

            let behaviour = if benchmark {
                QueryBehaviour::Benchmark {
                    num_runs: bench_num_runs,
                    warmup_runs: bench_num_warmup_runs,
                }
            } else {
                QueryBehaviour::Normal {
                    page_results,
                    write_results,
                    results_directory,
                    results_format,
                }
            };

            handle_query_cmd(query, behaviour, print_query).await?;
        }
    }

    Ok(())
}
