#[macro_use]
extern crate log;

use actix_web::{guard, middleware, web, App, HttpRequest, HttpServer, Responder};
use anyhow::Result;
use async_graphql::{Context, EmptySubscription, FieldResult, Object, Schema, ID};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use model::Coder;
use sqlx::postgres::PgPool;
use std::env;
use opentelemetry::global;
use tracing::{span, Subscriber};
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;


use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, Registry};
use tracing_subscriber::fmt::MakeWriter;

mod model;

async fn ping(_req: HttpRequest) -> impl Responder {
    format!(
        "I am healthy: {} v{}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION")
    )
}

struct Query;

#[Object(extends)]
impl Query {
    async fn coders<'a>(&self, ctx: &'a Context<'_>) -> FieldResult<Vec<Coder>> {
        let pool = ctx.data::<PgPool>().unwrap();
        let rows = Coder::read_all(&pool).await?;
        Ok(rows)
    }

    async fn coder<'a>(&self, ctx: &'a Context<'_>, id: String) -> FieldResult<Coder> {
        let pool = ctx.data::<PgPool>().unwrap();
        let row = Coder::read_one(&pool, &id).await?;
        Ok(row)
    }

    #[graphql(entity)]
    async fn find_coder_by_id<'a>(&self, ctx: &'a Context<'_>, id: String) -> FieldResult<Coder> {
        let pool = ctx.data::<PgPool>().unwrap();
        let row = Coder::read_one(&pool, &id).await?;
        Ok(row)
    }
}

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn create_coder(
        &self,
        ctx: &Context<'_>,
        title: String,
        description: String,
    ) -> FieldResult<Coder> {
        let pool = ctx.data::<PgPool>().unwrap();
        let row = Coder::create(&pool, &title, &description).await?;
        Ok(row)
    }

    async fn delete_coder(&self, ctx: &Context<'_>, id: ID) -> FieldResult<bool> {
        let pool = ctx.data::<PgPool>().unwrap();
        let id = id.parse::<String>()?;

        Coder::delete(&pool, &id).await?;
        Ok(true)
    }

    async fn update_coder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        description: String,
    ) -> FieldResult<Coder> {
        let pool = ctx.data::<PgPool>().unwrap();
        let id = id.parse::<String>()?;

        let row = Coder::update(&pool, &id, &description).await?;
        Ok(row)
    }
}

type ServiceSchema = Schema<Query, Mutation, EmptySubscription>;

async fn index(schema: web::Data<ServiceSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let subscriber = get_subscriber("svc-skill".into(), "info".into(), std::io::stdout);
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let host = env::var("HOST").expect("HOST is not set");
    let port = env::var("PORT").expect("PORT is not set");
    let db_pool = PgPool::connect(&database_url).await?;

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db_pool)
        .finish();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .wrap(TracingLogger::default())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .route("/ping", web::get().to(ping))
    });

    info!("Starting server");
    server.bind(format!("{}:{}", host, port))?.run().await?;

    Ok(())
}

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to spell out the actual
/// type of the returned subscriber, which is indeed quite complex.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
    where
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    // Install a new OpenTelemetry trace pipeline
    let tracer =     opentelemetry_zipkin::new_pipeline()
        .with_service_name("coder-rs")
        .install_batch(opentelemetry::runtime::Tokio).expect("");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);




    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(telemetry)
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}
