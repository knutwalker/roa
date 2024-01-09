use std::fmt::Display;

use kommandozeile::{
    clap,
    color_eyre::eyre::{ensure, OptionExt},
    concolor, pkg_name, setup_clap, setup_color_eyre_builder,
    tracing::debug,
    verbosity_filter, Color, Global, InputFile, OutputFile, Result, Verbose,
};
use secrecy::{ExposeSecret, SecretString};

use crate::dateformat::PublishDate;

/// mataroa.blog CLI
#[derive(Debug, clap::Parser)]
#[command(
    version(short_version()),
    long_version(long_version()),
    propagate_version(true),
    disable_help_subcommand(true),
    infer_long_args(true),
    infer_subcommands(true),
    arg_required_else_help(true)
)]
pub struct Args {
    #[clap(flatten)]
    api_keys: ApiKeys,

    #[clap(flatten)]
    verbose: Verbose<Global>,

    #[clap(flatten)]
    color: Color,

    /// Don't execute any requests, print their curl equivalent instead.
    #[clap(long, short = 'n')]
    pub dry_run: bool,

    /// Print the JSON response as is. Passing this flag will disable all
    /// other output options.
    #[clap(long, short)]
    pub json: bool,

    #[clap(skip)]
    pub use_color: bool,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Clone, Debug, clap::Args)]
#[group(multiple = false, required = true)]
struct ApiKeys {
    /// Read the API key from the given file.
    #[clap(long = "api-key-file", env = "MATAROA_API_KEY_FILE")]
    file: Option<InputFile>,

    /// Read the API key by calling the provided command.
    #[clap(long = "api-key-cmd", env = "MATAROA_API_KEY_CMD")]
    cmd: Option<String>,

    /// Read the API key from the given environment variable.
    #[clap(long = "api-key", env = "MATAROA_API_KEY")]
    key: Option<SecretString>,
}

impl Args {
    pub fn init() -> Result<Self> {
        let mut args = setup_clap::<Self>()
            .color_from(|a| a.color)
            .verbose_from(pkg_name!(), |a| a.verbose)
            .run();

        setup_color_eyre_builder()
            .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
            .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
            .install()?;

        args.use_color = concolor::get(concolor::Stream::Stdout).color();

        debug!(
            ?args,
            color = args.use_color,
            filter =% verbosity_filter!(args.verbose.verbosity()),
        );

        Ok(args)
    }

    pub fn api_key(&mut self) -> Result<SecretString> {
        let the_key: String;
        let api_key = match (
            self.api_keys.file.take(),
            self.api_keys.cmd.take(),
            self.api_keys.key.as_ref(),
        ) {
            (Some(file), None, None) => {
                the_key = file.read_to_string()?;
                the_key.trim()
            }
            (None, Some(cmd), None) => {
                let mut tokens = shlex::Shlex::new(&cmd);
                let cmd = tokens.next().ok_or_eyre("missing api-key command")?;
                let mut cmd = std::process::Command::new(cmd);
                let output = cmd.args(tokens).output()?;
                ensure!(
                    output.status.success(),
                    "api-key command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                the_key = String::from_utf8(output.stdout)?;
                the_key.trim()
            }
            (None, None, Some(key)) => key.expose_secret(),
            _ => unreachable!(),
        };
        let secret = SecretString::new(format!("Bearer {api_key}"));
        self.api_keys.key = None;
        Ok(secret)
    }
}

/// List all posts
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::Args)]
#[clap(visible_alias = "ls")]
pub struct List {
    /// Only print the slugs of the posts
    #[clap(long, short)]
    pub slugs: bool,
}

/// Get a post
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct Get {
    /// The slug of the post to get
    #[clap(required = true)]
    pub slug: String,

    #[clap(flatten)]
    pub print: GetPrint,
}

#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
#[group(multiple = false, required = false)]
pub struct GetPrint {
    /// Only print the body of the post
    #[clap(long, short)]
    pub body: bool,

    /// Don't print the body of the post
    #[clap(long, short)]
    pub no_body: bool,

    /// Write the post to the given file. The file will be a markdown with
    /// the body content and the frontmatter for the meta date.
    ///
    /// The file can be used as input to the `update` and `new` commands.
    ///
    /// The file will be overwritten if it already exists.
    #[clap(long, short, default_missing_value = "-")]
    pub output: Option<OutputFile>,
}

/// Update a post
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
#[clap(visible_alias = "edit")]
pub struct Update {
    /// The slug of the post to update
    #[clap(long, short)]
    pub slug: Option<String>,

    /// The file to update
    #[clap()]
    pub body: Option<InputFile>,

    /// The new slug for the post.
    /// If not provided, the slug will be read from the frontmatter of the file.
    /// If missing, the slug will not be updated.
    #[clap(long)]
    pub new_slug: Option<String>,

    /// The title of the post to update.
    /// If not provided, the title will be read from the frontmatter of the file.
    /// If missing, the title will not be updated.
    #[clap(long, short)]
    pub title: Option<String>,

    /// The published_at date of the post to update.
    /// If not provided, the published_at date will be read from the frontmatter of the file.
    /// If missing, the published_at date will not be updated.
    #[clap(long, short, value_parser = crate::dateformat::parse)]
    pub published_at: Option<PublishDate>,
}

/// Delete a post
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
#[clap(visible_alias = "rm")]
pub struct Delete {
    /// The slug of the post to delete
    #[clap()]
    pub slug: String,
}

/// Create a post
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
#[clap(visible_alias = "new")]
pub struct Create {
    /// The file to update
    #[clap()]
    pub body: InputFile,

    /// The title of the post to update.
    /// If not provided, the title will be read from the frontmatter of the file.
    /// If missing, the title will not be updated.
    #[clap(long, short)]
    pub title: Option<String>,

    /// The published_at date of the post to update.
    /// If not provided, the published_at date will be read from the frontmatter of the file.
    /// If missing, the published_at date will not be updated.
    #[clap(long, short, value_parser = crate::dateformat::parse)]
    pub published_at: Option<PublishDate>,
}

/// Run multiple commands in a row.
///
/// Provide a file with a list of commands to run.
/// One command per line.
/// The format of the file is the same as the CLI.
/// That is, each line is run as if called as "roa <..> $line".
/// Global options (such as --dry-run or --verbose, denoted as <..> above)
/// are taken from each individual incovation/line and not from the call
/// to the batch command.
///
/// Lines starting with # or ; are ignored.
/// Empty lines are ignored.
///
/// If a command fails, the batch will stop unless --ignore-errors is provided.
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct Batch {
    /// The batch file. See `batch --help` for details.
    #[clap()]
    pub batch: InputFile,

    /// Don't stop at the first error.
    ///
    /// Keep processing the batch. Errors will be logged buyt the overall
    /// execution will succeed at the end, even if all commands haved failed.
    #[clap(long)]
    pub ignore_errors: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, clap::Subcommand)]
pub enum Command {
    List(List),
    Create(Create),
    Get(Get),
    Update(Update),
    Delete(Delete),
    Batch(Batch),
}

const fn short_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn long_version() -> String {
    let info = Info::new();
    format!("\n{info}")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Info {
    build_version: &'static str,
    build_timestamp: &'static str,
    commit_sha: &'static str,
    commit_date: &'static str,
    rustc_version: &'static str,
    rustc_channel: &'static str,
    host_triple: &'static str,
    target_triple: &'static str,
    cargo_profile: &'static str,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<20} {}", "Build Version:", self.build_version)?;
        writeln!(f, "{:<20} {}", "Build Timestamp:", self.build_timestamp)?;
        writeln!(f, "{:<20} {}", "Commit SHA:", self.commit_sha)?;
        writeln!(f, "{:<20} {}", "Commit Date:", self.commit_date)?;
        writeln!(f, "{:<20} {}", "rustc Version:", self.rustc_version)?;
        writeln!(f, "{:<20} {}", "rustc Channel:", self.rustc_channel)?;
        writeln!(f, "{:<20} {}", "Host Triple:", self.host_triple)?;
        writeln!(f, "{:<20} {}", "Target Triple:", self.target_triple)?;
        writeln!(f, "{:<20} {}", "cargo Profile:", self.cargo_profile)?;
        Ok(())
    }
}

impl Info {
    const fn new() -> Self {
        Self {
            build_version: env!("CARGO_PKG_VERSION"),
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
            commit_sha: env!("VERGEN_GIT_SHA"),
            commit_date: env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
            rustc_version: env!("VERGEN_RUSTC_SEMVER"),
            rustc_channel: env!("VERGEN_RUSTC_CHANNEL"),
            host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
            target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
            cargo_profile: env!("VERGEN_CARGO_PROFILE"),
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert();
}
