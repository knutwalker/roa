use std::fmt::Debug;

use kommandozeile::{tracing::info, Result};
use secrecy::{ExposeSecret as _, SecretString};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use ureq::{Middleware, MiddlewareNext, Request, Response};

use crate::{dateformat::PublishDate, Post};

#[derive(Clone, Debug, TypedBuilder, Serialize)]
#[builder(doc)]
pub struct Create {
    title: String,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none", with = "crate::dateformat")]
    published_at: Option<PublishDate>,
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(doc)]
pub struct Get {
    slug: String,
}

#[derive(Clone, Debug, TypedBuilder, Serialize)]
#[builder(doc)]
pub struct Update {
    #[serde(skip)]
    slug: String,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[builder(default)]
    #[serde(rename = "slug", skip_serializing_if = "Option::is_none")]
    updated_slug: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none", with = "crate::dateformat")]
    published_at: Option<PublishDate>,
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(doc)]
pub struct Delete {
    slug: String,
}

#[derive(Clone, Debug, TypedBuilder, Serialize)]
#[builder(doc)]
pub struct List {}

pub struct Client {
    agent: ureq::Agent,
}

impl Client {
    pub fn new(api_key: SecretString) -> Self {
        let agent = ureq::AgentBuilder::new()
            .https_only(true)
            .no_delay(true)
            .redirects(2)
            .redirect_auth_headers(ureq::RedirectAuthHeaders::SameHost)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .middleware(AuthMiddleware { api_key })
            .build();

        Self { agent }
    }

    pub fn call<A: Action>(&self, action: A) -> Result<A::Res> {
        let response = self.request(&action)?;
        action.read(response)
    }

    pub fn print<A: Action>(&self, action: &A) -> Result<()> {
        let response = self.request(action)?;
        consume_response(response, std::io::stdout().lock())
    }

    pub fn dry_run<A: Action + Debug>(&self, action: &A) -> Result<()> {
        use std::fmt::Write as _;

        let request = action.request(&self.agent);
        info!(?action, ?request, "[dry-run]");

        let mut curl = String::with_capacity(1024);
        writeln!(curl, "curl -X {} \\", request.method())?;
        writeln!(
            curl,
            "  -H 'Authorization: Bearer <your mataroa API key goes here>' \\"
        )?;
        if let Some(body) = action.body() {
            let body = serde_json::to_string(&body)?;
            writeln!(curl, "  -d '{body}' \\")?;
        }
        write!(curl, "  {}", request.url())?;

        println!("{curl}");

        Ok(())
    }

    fn request<C: Action>(&self, call: &C) -> Result<Response> {
        let request = call.request(&self.agent);
        let response = match call.body() {
            Some(body) => request.send_json(body)?,
            None => request.call()?,
        };
        Ok(response)
    }
}

pub trait Action: Sized {
    fn request(&self, agent: &ureq::Agent) -> Request;

    fn body(&self) -> Option<&impl Serialize>;

    type Res;

    fn read(self, response: Response) -> Result<Self::Res>;
}

impl Action for Create {
    fn request(&self, agent: &ureq::Agent) -> Request {
        agent.post("https://mataroa.blog/api/posts/")
    }

    fn body(&self) -> Option<&impl Serialize> {
        Some(self)
    }

    type Res = Post;

    fn read(self, response: Response) -> Result<Self::Res> {
        let mut post = response.into_json::<Post>()?;
        post.title = Some(self.title);
        Ok(post)
    }
}

impl Action for Get {
    fn request(&self, agent: &ureq::Agent) -> Request {
        agent.get(&format!("https://mataroa.blog/api/posts/{}/", self.slug))
    }

    fn body(&self) -> Option<&impl Serialize> {
        None::<&String>
    }

    type Res = Post;

    fn read(self, response: Response) -> Result<Self::Res> {
        Ok(response.into_json::<Self::Res>()?)
    }
}

impl Action for Update {
    fn request(&self, agent: &ureq::Agent) -> Request {
        agent.patch(&format!("https://mataroa.blog/api/posts/{}/", self.slug))
    }

    fn body(&self) -> Option<&impl Serialize> {
        Some(self)
    }

    type Res = Post;

    fn read(self, response: Response) -> Result<Self::Res> {
        let mut post = response.into_json::<Post>()?;
        post.title = self.title;
        Ok(post)
    }
}

impl Action for Delete {
    fn request(&self, agent: &ureq::Agent) -> Request {
        agent.delete(&format!("https://mataroa.blog/api/posts/{}/", self.slug))
    }

    fn body(&self) -> Option<&impl Serialize> {
        None::<&String>
    }

    type Res = ();

    fn read(self, res: Response) -> Result<Self::Res> {
        consume_response(res, std::io::sink())?;
        Ok(())
    }
}

impl Action for List {
    fn request(&self, agent: &ureq::Agent) -> Request {
        agent.get("https://mataroa.blog/api/posts/")
    }

    fn body(&self) -> Option<&impl Serialize> {
        None::<&String>
    }

    type Res = Vec<Post>;

    fn read(self, response: Response) -> Result<Self::Res> {
        #[derive(Deserialize)]
        struct Posts {
            post_list: Vec<Post>,
        }

        Ok(response.into_json::<Posts>()?.post_list)
    }
}

struct AuthMiddleware {
    api_key: SecretString,
}

impl Middleware for AuthMiddleware {
    fn handle(&self, request: Request, next: MiddlewareNext<'_>) -> Result<Response, ureq::Error> {
        let request = request.set(
            "Authorization",
            &format!("Bearer {}", self.api_key.expose_secret()),
        );
        next.handle(request)
    }
}

fn consume_response<W: std::io::Write>(response: Response, mut sink: W) -> Result<()> {
    let mut read = response.into_reader();
    let _ = std::io::copy(&mut *read, &mut sink)?;
    Ok(())
}
