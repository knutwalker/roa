# mataroa-cli [![CI Status][ci-badge]][ci-url] [![Crates.io][crates-badge]][crates-url] [![Docs][docs-badge]][docs-url] ![License: MIT OR Apache-2.0][license-badge] ![Rust Version: none][rust-version-badge]

[ci-badge]: https://github.com/knutwalker/roa/actions/workflows/checks.yml/badge.svg
[ci-url]: https://github.com/knutwalker/roa
[crates-badge]: https://img.shields.io/crates/v/mataroa-cli?style=shield
[crates-url]: https://crates.io/crates/mataroa-cli
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/mataroa-cli
[license-badge]: https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=shield
[rust-version-badge]: https://img.shields.io/badge/rustc--orange.svg?style=shield

> [!NOTE]
> This CLI is not affiliated with mataroa and has no relation to
> https://github.com/mataroa-blog/mataroa-cli


### CLI for mataroa.blog

This is a CLI for [mataroa.blog][__link0], a naked blogging platform for minimalists.


## Installation


### From cargo:


```sh
cargo install mataroa-cli
```


### From source:


```sh
git clone https://github.com/knutwalker/roa.git
cd roa
make
sudo make install
```


## Usage


```sh
roa --help
```


### API Key

In order to use the CLI, you need an API key. You can get one by visiting loggin into mataroa and visting [https://mataroa.blog/api/docs/][__link1].

You can pass the API key in one of three ways (in order or preference):

 1. By providing a file that contains the key
 2. By providing a command that prints the key
 3. By providing the key directly


#### Providing a file

Set the `--api-key-file` flag to the path of the file. The file must contain only the API key as its content. A trailing newline is allowed.

This can also be set via the `MATAROA_API_KEY_FILE` environment variable. Using the flag will override the environment variable. Using the environment variable allows one to use the CLI without having to pass the API key every time.

An example file could be stored in `~/.config/mataroa/api-key`


#### Providing a command

Set the `--api-key-cmd` flag to the command that prints the API key. The command will be executed and its output will be used as the API key. The command must not produce any output other than the API key. A trailing newline is allowed.

This can also be set via the `MATAROA_API_KEY_CMD` environment variable. Using the flag will override the environment variable. Using the environment variable allows one to use the CLI without having to pass the API key every time.

An exmaple command could be `pass Mataroa/api-key`


#### Providing the key directly

Set the `--api-key` flag to the API key.

This can also be set via the `MATAROA_API_KEY` environment variable. Using the flag will override the environment variable. Using the environment variable allows one to use the CLI without having to pass the API key every time.

This is the least secure option, as the API key will be visible in the process list of the shell.


### Commands

The CLI supports the following commands (you can run `roa --help` to see them all, as well as `roa <command> --help` to see help for a specific command):

 - `list` or `ls`: List all posts
 - `create` or `new`: Create a post
 - `get`: Get a post
 - `update` or `edit`: Update a post
 - `delete` or `rm`: Delete a post


#### Command output

All commands will print a result meant for human consumption on the standard output.

In addition, all commands support the `--json` flag, which will print the JSON response from the mataroa API. Note that this is not identical to the regular output, as that one might do some post-processing on the result before printing it.

All commands also support a `--dry-run` flag, which will print the curl command that one could execute instead to achieve the same result as the command would do.


#### Date format

The `published_at` value can be provided in one of three ways:

 - As a date in the format `YYYY-MM-DD`
 - As a textual description, e.g. `tomorrow`, `next week`, `next month`, `today`, `now`
 - As the value `draft` or `none`

Setting the date to a future date will cause the post to be scheduled to publish at that date. Using the last option will send the empty string `""` as value, which will cause the post to be a draft. Often the `published_at` field is optional, so you can omit it entirely.


#### POST <> File mapping

Several commands accept a file as input. `get` also supports writing the result to a file.

That file is a markdown file representing the body of a post, with additional processing via a frontmatter.

The frontmatter is a YAML or JSON code block at the beginning of the file. It can contain the fields `title`, `slug`, or `published_at`. Depending on the command, some of these fields are required. Before the fromtmatter, there can be a leading h1, which will be used as the title for the blog post. An explicit title in the frontmatter overrides the h1 title.

The h1 and the frontmatter will be stripped from the document and are not part of the body. The body is everything after the frontmatter.

Example document:


```markdown
    # My first post
    
    ```yml
    published_at: "2015-10-21"
    ```
    
    This is the body of the post.
```

Using this file as input to `create` will create a post with the title `My first post`, the published date `2015-10-21`, and the body `This is the body of the post.`.

A file representing an existing post can be created by using the `get` command with the `--output` flag.


```sh
roa get --output post.md my-first-post
```

The files are not synced with the server, so if you update a post by other means than reading it from a file, you need to update the file manually (or run `get` again).

Commands that allow their values to be specified via a flag (e.g. `--title`) will use the value from the flag if it is provided, even if the file contains a value.


##### File usage per command


###### `create`

The file is required and must contain a title; either defined directly in the file as either h1 or as `title` in the frontmatter. It can also be provided via the `--title` flag.

The `published_at` value can be set and is optional.

The `slug` value will be ignored, as this one will be generated by the API.


###### `get`

A file can be created using the `--output` flag. If the file already exists, it will be overwritten. If the file is not provided, the post will be printed to stdout.

The file will contain an h1 title and a yml frontmatter with the `slug` and `published_at` fields, as well as the actual post body.


###### `update`

The file is optional and can be used to update the post. The `slug` must be provided either via the `--slug` flag or as the `slug` field in the frontmatter. The `slug` can also be changed using the `--new-slug` flag or by changing the content of the `slug` field in the frontmatter (in which case the slug to find the post to update needs to be provided via the `--slug` flag).

The `title`, `published_at`, and `body` fields are optional and can all be changed as well.


###### `delete` and `list`

Neither command uses a file.


#### Batch command

The CLI also supports a `batch` command, which allows you to run multiple commands at once. The commands are read from a file, one command per line. Empty lines and lines starting with `#`, `;` or `//` are ignored. The commands are parsed as if they were passed to the CLI directly.

Example:


```txt
list
new post.md
edit --slug post---pub draft
rm post
```



## License

mataroa-cli is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

---
 [__link0]: https://mataroa.blog
 [__link1]: https://mataroa.blog/api/docs/
