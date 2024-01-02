#![warn(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_copy_implementations,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_bounds,
    private_interfaces,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused_results
)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

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

use crate::{args::Command, dateformat::PublishDate};

mod api;
mod args;
mod dateformat;

fn main() -> Result<()> {
    let mut args = args::Args::init()?;
    let client = api::Client::new(args.api_key()?);
    cmd(&client, args.cmd, args.dry_run)
}

fn cmd(client: &api::Client, cmd: Command, dry_run: bool) -> Result<()> {
    match cmd {
        Command::List(cmd) => list(client, cmd, dry_run),
        Command::Create(cmd) => create(client, cmd, dry_run),
        Command::Get(cmd) => get(client, cmd, dry_run),
        Command::Update(cmd) => update(client, cmd, dry_run),
        Command::Delete(cmd) => delete(client, cmd, dry_run),
        Command::Batch(cmd) => batch(client, &cmd, dry_run),
    }
}

fn list(client: &api::Client, cmd: args::List, dry_run: bool) -> Result<()> {
    let action = api::List {};

    if dry_run {
        return client.dry_run(&action);
    }

    if cmd.print.json {
        return client.print(&action);
    }

    let posts = client.call(action)?;

    for post in posts {
        print!("{}", post.slug);
        if !cmd.print.slugs {
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
}

fn create(client: &api::Client, cmd: args::Create, dry_run: bool) -> Result<()> {
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

    let action = api::Create::builder()
        .title(title)
        .body(post.body)
        .published_at(post.published_at)
        .build();

    if dry_run {
        return client.dry_run(&action);
    }

    if cmd.json {
        return client.print(&action);
    }

    let post = client.call(action)?;

    print_post(post, true, false);

    Ok(())
}

fn get(client: &api::Client, cmd: args::Get, dry_run: bool) -> Result<()> {
    let action = api::Get::builder().slug(cmd.slug).build();

    if dry_run {
        return client.dry_run(&action);
    }

    if cmd.print.json {
        return client.print(&action);
    }

    let post = client.call(action)?;

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
}

fn update(client: &api::Client, cmd: args::Update, dry_run: bool) -> Result<()> {
    let post = PostInput::from(cmd.title, cmd.new_slug, cmd.published_at, cmd.body)?;
    let slug = cmd.slug.or_else(|| post.slug.clone()).ok_or_eyre(concat!(
        "The slug is required to update a post. ",
        "It can be provided via the --slug flag, ",
        "or as the slug key in the post frontmatter, ",
        "or as a leading h1 before the post frontmatter"
    ))?;

    let action = api::Update::builder()
        .slug(slug)
        .title(post.title)
        .updated_slug(post.slug)
        .body(post.body)
        .published_at(post.published_at)
        .build();

    if dry_run {
        return client.dry_run(&action);
    }

    if cmd.json {
        return client.print(&action);
    }

    let post = client.call(action)?;

    print_post(post, true, false);

    Ok(())
}

fn delete(client: &api::Client, cmd: args::Delete, dry_run: bool) -> Result<()> {
    let action = api::Delete::builder().slug(cmd.slug).build();

    if dry_run {
        return client.dry_run(&action);
    }

    if cmd.json {
        return client.print(&action);
    }

    client.call(action)?;

    Ok(())
}

fn batch(client: &api::Client, cmd: &args::Batch, dry_run: bool) -> Result<()> {
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
        .map(|args| args.and_then(|a| crate::cmd(client, a.cmd, dry_run)))
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
