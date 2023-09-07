use clap::Parser;
use clap::CommandFactory;
use clap::error::ErrorKind;

static PATH_PREFIX: &str = stringify!(./);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    command: String,

    #[arg(short, long, default_value_t=PATH_PREFIX.to_string())]
    path: String
}

pub enum ArgCommand {
    Generate(String),
    Run,
}
pub struct HydratedArgs {
    command: ArgCommand,
}
impl HydratedArgs {
    pub fn new(command: ArgCommand) -> Self {
        HydratedArgs { command: command }
    }

    pub fn get_command(&self) -> &ArgCommand { &self.command }
}

#[derive(Debug)]
pub struct ArgParseError;

pub fn parse_args() -> Result<HydratedArgs, ArgParseError> {
    let args = Args::parse();

    let command = match args.command.as_str() {
        "run" => Ok(ArgCommand::Run),
        "gen" => Ok(ArgCommand::Generate(args.path.to_string())),
        _ => {
            let mut cmd = Args::command();
            println!("{}", cmd.error(ErrorKind::InvalidValue, "Command provided is not valid"));
            let _ = cmd.print_help();
            panic!();
        }
    }?;

    let hydrated = HydratedArgs::new(command);
    Ok(hydrated)
}