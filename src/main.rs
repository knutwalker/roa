//! ## CLI for mataroa.blog
//!
//! This is a CLI for [mataroa.blog](https://mataroa.blog), a naked blogging platform for minimalists.
//!
//! # Installation
//!
//! ## From cargo:
//!
//! ```sh
//! cargo install mataroa-cli
//! ```
//!
//! ## From source:
//! ```sh
//! git clone https://github.com/knutwalker/roa.git
//! cd roa
//! make
//! sudo make install
//! ```
//!
//! # Usage
//!
//! ```sh
//! roa --help
//! ```
//! ## API Key
//!
//! In order to use the CLI, you need an API key.
//! You can get one by visiting loggin into mataroa and
//! visting [https://mataroa.blog/api/docs/](https://mataroa.blog/api/docs/).
//!
//! You can pass the API key in one of three ways (in order or preference):
//!
//! 1. By providing a file that contains the key
//! 2. By providing a command that prints the key
//! 3. By providing the key directly
//!
//! ### Providing a file
//!
//! Set the `--api-key-file` flag to the path of the file.
//! The file must contain only the API key as its content.
//! A trailing newline is allowed.
//!
//! This can also be set via the `MATAROA_API_KEY_FILE` environment variable.
//! Using the flag will override the environment variable.
//! Using the environment variable allows one to use the CLI without
//! having to pass the API key every time.
//!
//! An example file could be stored in `~/.config/mataroa/api-key`
//!
//! ### Providing a command
//!
//! Set the `--api-key-cmd` flag to the command that prints the API key.
//! The command will be executed and its output will be used as the API key.
//! The command must not produce any output other than the API key.
//! A trailing newline is allowed.
//!
//! This can also be set via the `MATAROA_API_KEY_CMD` environment variable.
//! Using the flag will override the environment variable.
//! Using the environment variable allows one to use the CLI without
//! having to pass the API key every time.
//!
//! An exmaple command could be `pass Mataroa/api-key`
//!
//! ### Providing the key directly
//!
//! Set the `--api-key` flag to the API key.
//!
//! This can also be set via the `MATAROA_API_KEY` environment variable.
//! Using the flag will override the environment variable.
//! Using the environment variable allows one to use the CLI without
//! having to pass the API key every time.
//!
//! This is the least secure option, as the API key will be visible in the
//! process list of the shell.
//!
//!
//! ## Commands
//!
//! The CLI supports the following commands (you can run `roa --help` to see them all, as well as
//! `roa <command> --help` to see help for a specific command):
//!
//! - `list` or `ls`: List all posts
//! - `create` or `new`: Create a post
//! - `get`: Get a post
//! - `update` or `edit`: Update a post
//! - `delete` or `rm`: Delete a post
//!
//! ### Command output
//!
//! All commands will print a result meant for human consumption on the standard output.
//!
//! In addition, all commands support the `--json` flag, which will print the JSON response
//! from the mataroa API.
//! Note that this is not identical to the regular output, as that one might do some
//! post-processing on the result before printing it.
//!
//! All commands also support a `--dry-run` flag, which will print the curl command that
//! one could execute instead to achieve the same result as the command would do.
//!
//! ### Date format
//!
//! The `published_at` value can be provided in one of three ways:
//! - As a date in the format `YYYY-MM-DD`
//! - As a textual description, e.g. `tomorrow`, `next week`, `next month`, `today`, `now`
//! - As the value `draft` or `none`
//!
//! Setting the date to a future date will cause the post to be scheduled to publish at that date.
//! Using the last option will send the empty string `""` as value, which will cause the post
//! to be a draft. Often the `published_at` field is optional, so you can omit it entirely.
//!
//! ### POST <> File mapping
//!
//! Several commands accept a file as input.
//! `get` also supports writing the result to a file.
//!
//! That file is a markdown file representing the body of a post, with additional
//! processing via a frontmatter.
//!
//! The frontmatter is a YAML or JSON code block at the beginning of the file.
//! It can contain the fields `title`, `slug`, or `published_at`.
//! Depending on the command, some of these fields are required.
//! Before the fromtmatter, there can be a leading h1, which will be used as the title for the blog post.
//! An explicit title in the frontmatter overrides the h1 title.
//!
//! The h1 and the frontmatter will be stripped from the document and are not part of the body.
//! The body is everything after the frontmatter.
//!
//! Example document:
//!
//! ```markdown
//!     # My first post
//!     
//!     ```yml
//!     published_at: "2015-10-21"
//!     ```
//!     
//!     This is the body of the post.
//! ```
//!
//! Using this file as input to `create` will create a post with the title `My first post`,
//! the published date `2015-10-21`, and the body `This is the body of the post.`.
//!
//! A file representing an existing post can be created by using the `get` command with the
//! `--output` flag.
//!
//! ```sh
//! roa get --output post.md my-first-post
//! ```
//!
//! The files are not synced with the server, so if you update a post by other means than
//! reading it from a file, you need to update the file manually (or run `get` again).
//!
//! Commands that allow their values to be specified via a flag (e.g. `--title`) will use
//! the value from the flag if it is provided, even if the file contains a value.
//!
//! #### File usage per command
//!
//! ##### `create`
//!
//! The file is required and must contain a title; either defined directly in the file
//! as either h1 or as `title` in the frontmatter. It can also be provided via the `--title` flag.
//!
//! The `published_at` value can be set and is optional.
//!
//! The `slug` value will be ignored, as this one will be generated by the API.
//!
//! ##### `get`
//!
//! A file can be created using the `--output` flag.
//! If the file already exists, it will be overwritten.
//! If the file is not provided, the post will be printed to stdout.
//!
//! The file will contain an h1 title and a yml frontmatter with the `slug` and `published_at`
//! fields, as well as the actual post body.
//!
//! ##### `update`
//!
//! The file is optional and can be used to update the post.
//! The `slug` must be provided either via the `--slug` flag or as the `slug` field in the frontmatter.
//! The `slug` can also be changed using the `--new-slug` flag or by changing the content of the
//! `slug` field in the frontmatter (in which case the slug to find the post to update needs to
//! be provided via the `--slug` flag).
//!
//! The `title`, `published_at`, and `body` fields are optional and can all be changed as well.
//!
//! ##### `delete` and `list`
//!
//! Neither command uses a file.
//!
//! ### Batch command
//!
//! The CLI also supports a `batch` command, which allows you to run multiple commands at once.
//! The commands are read from a file, one command per line.
//! Empty lines and lines starting with `#`, `;` or `//` are ignored.
//! The commands are parsed as if they were passed to the CLI directly.
//!
//! Example:
//! ```txt
//! # list all posts
//! list
//! # create a new post
//! new post.md
//! edit --slug post---pub draft
//! # delete the post
//! rm post
//! ```
use std::{fmt::Display, fs::File, io::Write as _};

use clap::Parser as _;
use kommandozeile::{
    color_eyre::eyre::{bail, OptionExt as _},
    tracing::warn,
    InputFile, OutputFile, Result,
};
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use serde::Deserialize;

use crate::{
    api::{Action, Opts},
    args::Command,
    dateformat::PublishDate,
};

mod api;
mod args;
mod dateformat;

fn main() -> Result<()> {
    let mut args = args::Args::init()?;
    let client = api::Client::new(args.api_key()?);
    run(&client, args)
}

fn run(client: &api::Client, args: args::Args) -> Result<()> {
    let opts = Opts::builder()
        .dry_run(args.dry_run)
        .print_json(args.json)
        .build();

    cmd(client, opts, args.cmd)
}

fn cmd(client: &api::Client, opts: Opts, cmd: Command) -> Result<()> {
    match cmd {
        Command::List(cmd) => list(client, opts, cmd),
        Command::Create(cmd) => create(client, opts, cmd),
        Command::Get(cmd) => get(client, opts, cmd),
        Command::Update(cmd) => update(client, opts, cmd),
        Command::Delete(cmd) => delete(client, opts, cmd),
        Command::Batch(cmd) => batch(client, &cmd),
    }
}

fn list(client: &api::Client, opts: Opts, cmd: args::List) -> Result<()> {
    api::List {}.run(client, opts, |posts| {
        for post in posts {
            print!("{}", post.slug);
            if !cmd.slugs {
                if let Some(title) = post.title {
                    print!(": {title}");
                }
                if let Some(url) = post.url {
                    print!(" ({url})");
                }
                if let Some(published_at) = post.published_at {
                    print!(" [{published_at}]");
                }
            }
            println!();
        }
        Ok(())
    })
}

fn create(client: &api::Client, opts: Opts, cmd: args::Create) -> Result<()> {
    let post = PostInput::from(cmd.title, None, cmd.published_at, Some(cmd.body))?;
    if post.slug.is_some() {
        warn!(concat!(
            "The slug is ignored when creating a new post. ",
            "If the created slug differed from the one provided, ",
            "future updates of the post will change that slug."
        ));
    }

    let title = post.title.ok_or_eyre(concat!(
        "The title is required to create a new post. ",
        "It can be provided via the --title flag, ",
        "or as the title key in the post frontmatter, ",
        "or as a leading h1 before the post frontmatter"
    ))?;

    api::Create::builder()
        .title(title)
        .body(post.body)
        .published_at(post.published_at)
        .build()
        .run(client, opts, |post| {
            print_post(post, true, false);
            Ok(())
        })
}

fn get(client: &api::Client, opts: Opts, cmd: args::Get) -> Result<()> {
    api::Get::builder()
        .slug(cmd.slug)
        .build()
        .run(client, opts, |post| {
            if let Some(file) = cmd.print.output {
                match file {
                    OutputFile::File(path) | OutputFile::Stdout(Some(path)) => {
                        let mut file = File::create(path)?;
                        write!(file, "{post}")?;
                        file.flush()?;
                    }
                    OutputFile::Stdout(None) => {
                        let mut stdout = std::io::stdout().lock();
                        write!(stdout, "{post}")?;
                        stdout.flush()?;
                    }
                }
                return Ok(());
            }

            print_post(post, !cmd.print.body, !cmd.print.no_body);
            Ok(())
        })
}

fn update(client: &api::Client, opts: Opts, cmd: args::Update) -> Result<()> {
    let post = PostInput::from(cmd.title, cmd.new_slug, cmd.published_at, cmd.body)?;
    let slug = cmd.slug.or_else(|| post.slug.clone()).ok_or_eyre(concat!(
        "The slug is required to update a post. ",
        "It can be provided via the --slug flag, ",
        "or as the slug key in the post frontmatter.",
    ))?;

    api::Update::builder()
        .slug(slug)
        .title(post.title)
        .updated_slug(post.slug)
        .body(post.body)
        .published_at(post.published_at)
        .build()
        .run(client, opts, |post| {
            print_post(post, true, false);
            Ok(())
        })
}

fn delete(client: &api::Client, opts: Opts, cmd: args::Delete) -> Result<()> {
    api::Delete::builder()
        .slug(cmd.slug)
        .build()
        .run(client, opts, |()| Ok(()))
}

fn batch(client: &api::Client, cmd: &args::Batch) -> Result<()> {
    let batch = cmd.batch.read_to_string()?;
    batch
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with('#')
                || line.starts_with(';')
                || line.starts_with("//")
            {
                None
            } else {
                Some(line)
            }
        })
        .map(|line| {
            Ok(args::Args::try_parse_from(
                std::iter::once("batch".to_owned()).chain(shlex::Shlex::new(line)),
            )?)
        })
        .map(|args| args.and_then(|a| run(client, a)))
        .try_for_each(|result| {
            if cmd.ignore_errors {
                if let Err(err) = result {
                    warn!(?err, "Batch error");
                    return Ok(());
                }
            }
            result
        })?;

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Post {
    pub slug: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub url: Option<String>,
}

fn print_post(post: Post, meta: bool, body: bool) {
    if meta {
        print!("{}", post.slug);
        if let Some(title) = post.title {
            print!(": {title}");
        }
        println!();
        if let Some(url) = post.url {
            println!("URL: {url}");
        }
        if let Some(published_at) = post.published_at {
            println!("Published at: {published_at}");
        }
    }

    if body {
        if let Some(body) = post.body {
            println!("{body}");
        }
    }
}

impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(title) = &self.title {
            writeln!(f, "# {title}")?;
            writeln!(f)?;
        }

        writeln!(f, "```yml")?;
        writeln!(f, "slug: {}", self.slug)?;
        let published_at = self.published_at.as_deref().unwrap_or_default();
        writeln!(f, "published_at: \"{published_at}\"")?;
        writeln!(f, "```")?;
        writeln!(f)?;

        if let Some(body) = &self.body {
            writeln!(f, "{body}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PostInput {
    title: Option<String>,
    slug: Option<String>,
    #[serde(default, with = "dateformat")]
    published_at: Option<PublishDate>,
    #[serde(skip)]
    body: Option<String>,
}

impl PostInput {
    fn from(
        title: Option<String>,
        slug: Option<String>,
        published_at: Option<PublishDate>,
        body: Option<InputFile>,
    ) -> Result<Self> {
        let mut post = Self::new(title, slug, published_at);
        if let Some(body) = body {
            post.update(&body)?;
        }
        Ok(post)
    }

    const fn new(
        title: Option<String>,
        slug: Option<String>,
        published_at: Option<PublishDate>,
    ) -> Self {
        Self {
            title,
            slug,
            published_at,
            body: None,
        }
    }

    fn update(&mut self, file: &InputFile) -> Result<()> {
        let new = Self::from_file(file)?;
        if self.title.is_none() {
            self.title = new.title;
        }
        if self.slug.is_none() {
            self.slug = new.slug;
        }
        if self.published_at.is_none() {
            self.published_at = new.published_at;
        }
        if self.body.is_none() {
            self.body = new.body;
        }
        Ok(())
    }

    fn from_file(file: &InputFile) -> Result<Self> {
        let file = file.read_to_string()?;
        Self::from_markdown(&file)
    }

    fn from_markdown(content: &str) -> Result<Self> {
        enum Lang {
            Yaml,
            Json,
        }

        let mut parser = Parser::new_ext(content, Options::all());

        let fm_parser = parser.by_ref();

        let extractor = FrontmatterExtractor::new(fm_parser);
        let frontmatter = extractor.extract();

        let rest = parser.into_offset_iter().next();
        let rest = rest.map_or(content.len(), |(_, rest)| rest.start);
        let body = Some(content[rest..].trim().to_owned()).filter(|s| !s.is_empty());

        let Some(frontmatter) = frontmatter else {
            warn!("No frontmatter detected");
            return Ok(Self {
                body,
                ..Self::default()
            });
        };

        let Some(cb) = frontmatter.code_block else {
            warn!("No code block detected");
            return Ok(Self {
                body,
                title: frontmatter.title,
                ..Self::default()
            });
        };

        let lang = match cb.language {
            None => Lang::Yaml,
            Some(l) if l.as_ref() == "yaml" => Lang::Yaml,
            Some(l) if l.as_ref() == "yml" => Lang::Yaml,
            Some(l) if l.as_ref() == "json" => Lang::Json,
            Some(l) => bail!("'{}' is not supported, only yaml and json are.", l),
        };

        let meta: Self = match lang {
            Lang::Yaml => serde_yaml::from_str(&cb.source)?,
            Lang::Json => serde_json::from_str(&cb.source)?,
        };

        Ok(Self {
            body,
            title: meta.title.or(frontmatter.title),
            ..meta
        })
    }
}
