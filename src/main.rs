mod config;
mod task;
use clap::Parser;
use config::StagedTask;
use task::stream_command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Args {
    command: String,
    #[arg(default_value = "config.yaml", long)]
    config: String,
    #[arg(trailing_var_arg = true)]
    task_args: Vec<String>,
}

fn load_task_file(file_path: String) -> Result<config::Config, std::io::Error> {
    let f = std::fs::File::open(file_path).expect("Could not open file");
    let d: config::Config = serde_yaml::from_reader(f).expect("Could not read values.");
    Ok(d)
}

fn main() {
    let args = Args::parse();
    let config: config::Config = load_task_file(args.config).unwrap();
    let selected_command = match config.get_task_from_name(&args.command) {
        Some(tsk) => tsk,
        None => {
            show_help_statement(&config);
            std::process::exit(1)
        }
    };
    let staged: StagedTask = StagedTask {
        selected_command: selected_command,
        command_inputs: args.task_args,
    };
    let _ = stream_command(staged);
}

fn show_help_statement(config: &config::Config) {
    println!("Help! {:?}", config)
}
