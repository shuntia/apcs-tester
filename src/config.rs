use crate::checker;
use crate::executable::Language;
use clap::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::LazyCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use std::sync::LazyLock;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
fn load_config() -> Config {
    let cp: ConfigParams =
        match ARGS.get_config().unwrap() {
            s if s.extension().expect(
                "File extension not found! Config format guessing is not implemented yet!",
            ) == "json" =>
            {
                serde_json::from_reader(File::open(s).unwrap())
                    .expect("Illegal config! Failed to parse JSON!")
            }
            s if s.extension().expect(
                "File extension not found! Config format guessing is not implemented yet!",
            ) == "toml" =>
            {
                let mut string = String::new();
                let _ = File::open(s).unwrap().read_to_string(&mut string);
                toml::from_str(string.as_str()).expect("Illegal config! Failed to parse JSON!")
            }
            _ => {
                panic!("File extension not found! Config format guessing is not implemented yet!");
            }
        };
    if cp.target == None {
        error!("What do you mean target is none? Why are you running this program!?");
        exit(1);
    }
    let config = Config {
        entry: cp.entry,
        lang: Language::Guess,
        target: cp.target.unwrap_or(std::env::current_dir().unwrap()),
        args: cp.args.unwrap_or(vec![]),
        input: cp.input.unwrap_or(vec![]),
        output: cp.output.unwrap_or(vec![]),
        points: cp.points.unwrap_or(vec![]),
        timeout: cp.timeout.unwrap_or(5),
        memory: cp.memory.unwrap_or(1024),
        threads: cp.threads.unwrap_or(4),
        checker: cp
            .checker
            .map(|x| match x.as_str() {
                "static" => checker::Type::Static,
                "ast" => checker::Type::AST,
                _ => checker::Type::AST,
            })
            .unwrap(),
        allow: cp.allow.unwrap_or(vec![]),
        format: match &cp.format {
            Some(s) => s.into(),
            None => "{name}_{num}_{num}.{ext}".into(),
        },
        orderby: cp.orderby.unwrap_or(Orderby::Id),
    };
    if config.input.len() != config.output.len() {
        warn!("CONFIG: potential misalignment in input-output pair.");
    };
    config
}

pub fn get_config() -> Result<&'static Lazy<Config>, String> {
    Ok(&CONFIG)
}

pub fn generate_regex(format: &str) -> Regex {
    // Predefined placeholders and their regex patterns
    let placeholders = HashMap::from([
        ("name", "(?P<name>[a-zA-Z][a-zA-Z0-9_]*)"), // Starts with a letter, allows alnum + underscore
        ("alpha", "(?P<alpha>[a-zA-Z]+)"),           // Only letters
        ("num", "(?P<num>\\d+)"),                    // Only numbers
        ("alnum", "(?P<alnum>[a-zA-Z0-9]+)"),        // Letters & numbers
        ("word", "(?P<word>\\w+)"),                  // Word (letters, numbers, underscore)
        ("id", "(?P<id>\\d+)"),                      // Numeric ID
        ("extension", "(?P<extension>\\w+)"),        // File extension (word characters)
    ]);

    // Replace placeholders with corresponding regex patterns
    let mut pattern = format.to_string();
    for (key, value) in &placeholders {
        pattern = pattern.replace(&format!("{{{}}}", key), value);
    }

    // Escape the dot (.) for file extensions
    pattern = pattern.replace(".", "\\.");

    Regex::new(&format!("^{}$", pattern)).unwrap()
}
pub fn from(value: String) -> Language {
    match value.as_str() {
        "java" => Language::Java,
        "jar" => Language::Java,
        "cpp" => Language::Cpp,
        "c" => Language::C,
        "rs" => Language::Rust,
        "py" => Language::Python,
        _ => Language::Unknown("".into()),
    }
}

#[deprecated]
pub fn match_ext(s: &str) -> Language {
    from(s.to_owned())
}

pub static TEMPDIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/tmp/"));

pub static CONFIG: Lazy<Config> = Lazy::new(load_config);

#[derive(Serialize, Deserialize)]
struct ConfigParams {
    entry: Option<String>,
    lang: Option<String>,
    args: Option<Vec<String>>,
    target: Option<PathBuf>,
    input: Option<Vec<Vec<String>>>,
    output: Option<Vec<String>>,
    points: Option<Vec<u64>>,
    timeout: Option<u64>,
    memory: Option<u64>,
    threads: Option<u64>,
    checker: Option<String>,
    allow: Option<Vec<String>>,
    format: Option<String>,
    orderby: Option<Orderby>,
}

#[derive(Clone, Serialize)]
pub struct Config {
    pub entry: Option<String>,
    pub lang: Language,
    pub args: Vec<String>,
    pub target: PathBuf,
    pub input: Vec<Vec<String>>,
    pub output: Vec<String>,
    pub points: Vec<u64>,
    pub timeout: u64,
    pub memory: u64,
    pub threads: u64,
    pub checker: checker::Type,
    pub allow: Vec<String>,
    pub format: String,
    pub orderby: Orderby,
}
#[derive(Clone, Serialize, Deserialize)]
pub enum Orderby {
    Name,
    Id,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            entry: None,
            lang: Language::Guess,
            args: vec![],
            target: PathBuf::new(),
            input: vec![],
            output: vec![],
            points: vec![],
            timeout: 500,
            memory: 10,
            threads: 5,
            checker: checker::Type::AST,
            allow: vec![],
            format: "{name}_{num}_{num}.{ext}".into(),
            orderby: Orderby::Id,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Language: {:?}", self.lang)?;
        writeln!(f, "Args: {:?}", self.args)?;
        writeln!(f, "Target: {:?}", self.target)?;
        writeln!(f, "Input: {:?}", self.input)?;
        writeln!(f, "Output: {:?}", self.output)?;
        writeln!(f, "Points: {:?}", self.points)?;
        writeln!(f, "Timeout: {:?}", self.timeout)?;
        writeln!(f, "Memory: {:?}MB", self.memory)?;
        writeln!(f, "Threads: {:?}", self.threads)?;
        writeln!(f, "Checker: {:?}", self.checker)?;
        writeln!(f, "Allow: {:?}", self.allow)
    }
}

#[derive(Parser, Clone)]
pub struct Args {
    /// subcommands
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    Init,
    Run,
    Test,
    Format,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// initialize the tests
    Init {
        /// do not output any logs except for panic(fatal errors)
        #[clap(short, long)]
        silent: bool,
        /// do not output any logs except for errors
        #[clap(short, long)]
        quiet: bool,
    },
    /// run the tests
    Run {
        /// Test functionality
        #[clap(short, long)]
        test: Option<String>,
        /// verbose mode
        #[clap(short, long)]
        verbose: bool,
        /// debug mode
        #[clap(long)]
        debug: bool,
        /// quiet mode
        #[clap(short, long)]
        quiet: bool,
        /// silent mode
        #[clap(short, long)]
        silent: bool,
        /// log level
        #[clap(short, long)]
        log_level: Option<u32>,
        /// configuration file for tests
        #[clap(long)]
        config: Option<PathBuf>,
        /// input file or directory
        #[clap(short, long)]
        input: Option<PathBuf>,
        /// output file or directory for results
        #[clap(short, long)]
        output: Option<PathBuf>,
        /// dry-run and just execute, don't input anything.
        #[clap(long)]
        dry_run: bool,
        /// leave artifacts
        #[clap(long, short)]
        artifacts: bool,
    },
    /// test features
    Test,
    Format,
}

impl Args {
    pub fn get_config(&self) -> Option<&PathBuf> {
        match &self.command {
            Command::Run {
                test,
                verbose,
                debug,
                quiet,
                silent,
                log_level,
                config,
                input,
                output,
                dry_run,
                artifacts,
            } => config.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleOpts {
    pub mode: CommandType,
    /// Test functionality
    pub test: Option<String>,
    /// verbose mode
    pub verbose: bool,
    /// debug mode
    pub debug: bool,
    /// quiet mode
    pub quiet: bool,
    /// silent mode
    pub silent: bool,
    /// log level
    pub log_level: Option<u32>,
    /// configuration file for tests
    pub config: Option<PathBuf>,
    /// input file or directory
    pub input: Option<PathBuf>,
    /// output file or directory for results
    pub output: Option<PathBuf>,
    /// dry-run and just execute, don't input anything.
    pub dry_run: bool,
    /// leave artifacts
    pub artifacts: bool,
}
impl SimpleOpts {
    pub fn new() -> Self {
        (*ARGS).clone().into()
    }
}

impl Default for SimpleOpts {
    fn default() -> Self {
        SimpleOpts {
            mode: CommandType::Init,
            test: None,
            verbose: false,
            debug: false,
            quiet: false,
            silent: false,
            log_level: None,
            config: None,
            input: None,
            output: None,
            dry_run: true,
            artifacts: false,
        }
    }
}

impl From<Lazy<Args>> for SimpleOpts {
    fn from(value: Lazy<Args>) -> Self {
        let mut ret = SimpleOpts::default();
        match &value.command {
            Command::Init { silent, quiet } => {
                ret.quiet = quiet.clone();
                ret.silent = silent.clone();
            }
            Command::Run {
                test,
                verbose,
                debug,
                quiet,
                silent,
                log_level,
                config,
                input,
                output,
                dry_run,
                artifacts,
            } => {
                ret.test = test.clone();
                ret.verbose = verbose.clone();
                ret.debug = debug.clone();
                ret.quiet = quiet.clone();
                ret.silent = silent.clone();
                ret.log_level = log_level.clone();
                ret.config = config.clone();
                ret.input = input.clone();
                ret.output = output.clone();
                ret.dry_run = dry_run.clone();
                ret.artifacts = artifacts.clone();
            }
            _ => {}
        }
        return ret;
    }
}
impl From<LazyCell<Args>> for SimpleOpts {
    fn from(value: LazyCell<Args>) -> Self {
        let mut ret = SimpleOpts::default();
        match &value.command {
            Command::Init { silent, quiet } => {
                ret.quiet = quiet.clone();
                ret.silent = silent.clone();
            }
            Command::Run {
                test,
                verbose,
                debug,
                quiet,
                silent,
                log_level,
                config,
                input,
                output,
                dry_run,
                artifacts,
            } => {
                ret.test = test.clone();
                ret.verbose = verbose.clone();
                ret.debug = debug.clone();
                ret.quiet = quiet.clone();
                ret.silent = silent.clone();
                ret.log_level = log_level.clone();
                ret.config = config.clone();
                ret.input = input.clone();
                ret.output = output.clone();
                ret.dry_run = dry_run.clone();
                ret.artifacts = artifacts.clone();
            }
            _ => {}
        }
        return ret;
    }
}

impl From<Args> for SimpleOpts {
    fn from(value: Args) -> Self {
        let mut ret = SimpleOpts::default();
        match value.command {
            Command::Init { silent, quiet } => {
                ret.quiet = quiet;
                ret.silent = silent;
            }
            Command::Run {
                test,
                verbose,
                debug,
                quiet,
                silent,
                log_level,
                config,
                input,
                output,
                dry_run,
                artifacts,
            } => {
                ret.test = test;
                ret.verbose = verbose;
                ret.debug = debug;
                ret.quiet = quiet;
                ret.silent = silent;
                ret.log_level = log_level;
                ret.config = config;
                ret.input = input;
                ret.output = output;
                ret.dry_run = dry_run;
                ret.artifacts = artifacts;
            }
            _ => {}
        }
        return ret;
    }
}

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);
pub static SIMPLEOPTS: Lazy<SimpleOpts> = Lazy::new(SimpleOpts::new);

pub fn proc_args() {
    match &ARGS.command {
        Command::Init { silent, quiet } => {}
        Command::Run {
            test,
            verbose,
            debug,
            quiet,
            silent,
            log_level,
            config,
            input,
            output,
            dry_run,
            artifacts,
        } => {
            if *test != None {
                println!("Test mode is enabled. Ignoring rest of arguments.");
            }
            if *verbose {
                info!("Verbose mode enabled");
            };
            if *debug {
                debug!("Debug mode enabled");
            };
            if *config == None {
                error!("No configuration file specified! The program will attempt to find one inside the target directory.");
            };
            if *input == None {
                error!("No input file or directory specified");
                if *config == None {
                    error!("No input directory nor config file! Tester does not know what to do!");
                    panic!("Unable to run anything!");
                }
            } else if input.clone().unwrap().is_file() {
                if *config == None {
                    panic!("Cannot probe config file with only one provided file.");
                }
            }
            if *output == None {
                info!("No output file or directory specified. falling back to stdout.");
            } else {
                let tmp = output.clone().unwrap();
                if tmp.is_dir() {
                    unimplemented!("Output is a directory! Not supported yet.");
                } else {
                    info!("Output file: {}", tmp.display());
                    match tmp
                        .extension()
                        .expect("Expected file format!")
                        .to_str()
                        .unwrap()
                    {
                        "json" => {
                            info!("Output format: JSON");
                        }
                        "txt" => {
                            info!("Output format: Plaintext");
                        }
                        _ => {
                            error!(
                                "Unsupported output format: {}",
                                tmp.extension()
                                    .expect("Expected file format!")
                                    .to_str()
                                    .unwrap()
                            );
                            info!("falling back to stdout.");
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
