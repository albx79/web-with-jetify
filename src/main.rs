mod utils;

use std::fmt::Debug;
use std::sync::{Arc};
use anyhow::Context;
use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Router, Form};
use axum::extract::{Path, Query, State};
use axum::routing::post;
use edgedb_protocol::model::Uuid;
use edgedb_tokio::Client as EdgeClient;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::utils::print_edgedb_err;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client: EdgeClient = edgedb_tokio::create_client().await?;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let api_router = Router::new()
        .route("/todos", post(add_todo::<EdgeClient>))
        .route("/hello", get(hello_from_the_server))
        .with_state(client.clone());

    let fate_router = Router::new()
        .route("/characters/:id", get(render_character))
        .with_state(client.clone());

    let router = Router::new()
        .nest("/api", api_router)
        .nest("/fate", fate_router)
        .route("/", get(hello::<EdgeClient>))
        .route("/another-page", get(another_page))
        .with_state(client);
    let port = 8080_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    info!("router initialized, now listening on port {}", port);

    axum::serve(tokio::net::TcpListener::bind(&addr).await?, router).await?;
    Ok(())
}

struct AppState {
    todos: Mutex<Vec<String>>,
}

async fn hello<T: TodoStore>(State(todos): State<T>) -> impl IntoResponse {
    let template = HelloTemplate {
        todos: todos.all_todos().await.expect("Cannot get todos")
    };
    HtmlTemplate(template)
}

async fn another_page() -> impl IntoResponse {
    HtmlTemplate(AnotherPageTemplate {})
}

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

#[derive(Template, Debug)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

trait TodoStore: Send {
    async fn add_todo(&self, todo: String) -> anyhow::Result<Uuid>;
    async fn all_todos(&self) -> anyhow::Result<Vec<String>>;
}

impl TodoStore for Arc<AppState> {
    async fn add_todo(&self, todo: String) -> anyhow::Result<Uuid> {
        let res = Uuid::new_v4();
        let mut lock = self.todos.lock().await;
        lock.push(todo);
        Ok(res)
    }

    async fn all_todos(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.todos.lock().await.clone())
    }
}

impl TodoStore for EdgeClient {
    async fn add_todo(&self, todo: String) -> anyhow::Result<Uuid> {
        let id: Uuid = self.query_required_single(r##"
            with
                msg := <str>$0,
                new := (insert Todos { todo := msg })
            select new.id
        "##, &(todo, )).await?;
        Ok(id)
    }

    async fn all_todos(&self) -> anyhow::Result<Vec<String>> {
        let todos: Vec<String> = self.query("select Todos.todo", &()).await?;
        Ok(todos)
    }
}

async fn add_todo<T: TodoStore>(
    State(state): State<T>,
    Form(todo): Form<TodoRequest>,
) -> impl IntoResponse {
    state.add_todo(todo.todo).await.expect("Could not save TODO");

    let template = TodoList {
        todos: state.all_todos().await.expect("Cannot get todos")
    };

    HtmlTemplate(template)
}

#[derive(Serialize, Deserialize)]
struct TodoRequest {
    todo: String,
}

#[derive(Template, Debug)]
#[template(path = "hello.html")]
struct HelloTemplate {
    todos: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);


#[derive(Template, Debug)]
#[template(path = "character-sheet.html")]
struct CharacterSheet {
    character: Character,
    all_skills: Vec<String>,
    editable: bool,
}

#[derive(Deserialize, Debug)]
struct Character {
    name: String,
    aspects: Vec<String>,
    skills: Vec<Skill>,
    stunts: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Skill {
    name: String,
    rating: u8,
}

#[derive(Deserialize, Default)]
struct RenderCharacter {
    #[serde(default)]
    editable: bool,
}


mod dto {
    use edgedb_derive::Queryable;
    use serde::Deserialize;

    #[derive(Deserialize, Queryable)]
    pub struct Character {
        pub name: String,
        pub stunts: Vec<String>,
        pub skills: Vec<SkillOuter>,
        pub aspects: Vec<Aspect>,
    }

    #[derive(Deserialize, Queryable)]
    pub struct SkillOuter {
        pub name: SkillInner,
        pub level: i32,
    }

    #[derive(Deserialize, Queryable)]
    pub struct SkillInner {
        pub name: String,
    }

    #[derive(Deserialize, Queryable)]
    pub struct Aspect {
        pub description: String,
        pub aspect_type: AspectType,
    }

    #[derive(Deserialize, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Queryable)]
    pub enum AspectType {
        High,
        Trouble,
        Other
    }
}

async fn render_character(Path(id): Path<String>, Query(params): Query<RenderCharacter>, State(client): State<EdgeClient>) -> impl IntoResponse {
    // let mut c: dto::Character =
    let c = client.query_required_single_json(r##"
        select fate::PC {
            name,
            stunts,
            skills: { name: { name }, level },
            aspects: { description, aspect_type },
        }
        filter .id = <uuid><str>$0
    "##, &(id,)).await
        .map_err(print_edgedb_err)
        .unwrap();
    let mut c: dto::Character = serde_json::from_str(&c).with_context(|| c.to_string()).unwrap();
    c.aspects.sort_by_key(|a| a.aspect_type);
    let skills = client.query_json("select fate::AllowedSkill { name } order by.name", &())
        .await
        .map_err(print_edgedb_err)
        .unwrap();
    let all_skills: Vec<dto::SkillInner> = serde_json::from_str(&skills).with_context(|| skills.to_string()).unwrap();
    let char = CharacterSheet {
        character: Character {
            name: c.name,
            aspects: c.aspects.into_iter().map(|a| a.description).collect(),
            skills: c.skills.into_iter().map(|s| Skill {
                name: s.name.name,
                rating: s.level as u8
            }).collect(),
            stunts: c.stunts,
        },
        all_skills: all_skills.into_iter().map(|s| s.name).collect(),
        editable: params.editable,
    };
    HtmlTemplate(char)
}

#[derive(Debug, Deserialize)]
struct StuntReq {
    stunts: Vec<String>
}

pub fn save(Path(char_id): Path<String>, Form(changed): Form<Character> ) -> impl IntoResponse {
    let query = ();
    unimplemented!()

}

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template + Debug,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}; data {:?}", err, &self.0),
            )
                .into_response(),
        }
    }
}
