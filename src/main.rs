mod config;
mod task;
use clap::Parser;
use config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Args {
    command: String,
    #[arg(default_value = "config.yaml", long)]
    config: String,
    #[arg(trailing_var_arg = true)]
    task_args: Vec<String>,
}

fn main() {
    // let args = Args::parse();
    // let mut cmd = Args::command();
    // cmd.print_help();
    let config = Config::new("config.yaml".to_string()).unwrap();
    let cmd = config.create_clap_command();
    let inputs = cmd.get_matches();
    // we can be confident in unwraps since CLAP can handle most errors on read above
    let subcmd = inputs.subcommand_name().unwrap();
    let chosen_command = config.get_task_by_name(subcmd).unwrap();
    let (_, subcmd_struct) = inputs.subcommand().unwrap();
    let subcmd_inputs = subcmd_struct.to_owned();
    let _ = chosen_command.stream_command(subcmd_inputs);

    // let staged: StagedTask = StagedTask {
    //     selected_command: chosen_command,
    //     command_inputs: inputs.get_raw(id)
    // };
    // println!("{:?}", x)
    // let selected_command = match config.get_task_by_name(&args.command) {
    //     Some(tsk) => tsk,
    //     None => {
    //         &config.show_help_statement();
    //         std::process::exit(1)
    //     }
    // };
    // let staged: StagedTask = StagedTask {
    //     selected_command: selected_command,
    //     command_inputs: args.task_args,
    // };
    // let _ = staged.stream_command();
}
