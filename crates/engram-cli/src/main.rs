use clap::Parser;
use engram_cli::{Cli, run, output};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let fmt = output::OutputFormat::from_flags(cli.json);
    match run(cli, fmt).await {
        Ok(()) => {}
        Err(err) => {
            output::print_error(&err, fmt);
            std::process::exit(output::error_exit_code(&err));
        }
    }
}

