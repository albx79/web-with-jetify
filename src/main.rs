use std::sync::{Arc};
use anyhow::Context;
use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Router, Form};
use axum::extract::State;
use axum::routing::post;
use edgedb_protocol::model::Uuid;
use edgedb_tokio::Client as EdgeClient;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

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
        .with_state(client.clone()); //Arc::new(AppState { todos: Mutex::new(Vec::new())}));
    let router = Router::new()
        .nest("/api", api_router)
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
        todos: todos.all_todos().await.expect("Cannot get todos")//.into_iter().map(|(s)| s).collect()
    };
    HtmlTemplate(template)
}

async fn another_page() -> impl IntoResponse {
    HtmlTemplate(AnotherPageTemplate {})
}

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

#[derive(Template)]
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
        todos: state.all_todos().await.expect("Cannot get todos")//.into_iter().map(|(_, s)| s).collect(),
    };

    HtmlTemplate(template)
}

#[derive(Serialize, Deserialize)]
struct TodoRequest {
    todo: String,
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    todos: Vec<String>,
}

#[derive(Template)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}