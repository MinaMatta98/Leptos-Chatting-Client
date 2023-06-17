#![feature(let_chains)]
#![feature(async_closure)]
use actix_web::{
    cookie::Key, dev, get, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use futures_util::future;
pub use sea_orm::{Database, DbErr, *};
pub mod app;
pub mod database;
pub mod emailing;
pub mod entities;
pub mod migrator;
pub mod server_function;
use app::*;
use database::{database_run, DbConnection};

async fn identity_middleware(
    req: HttpRequest,
    id: actix_identity::Identity,
) -> Result<dev::ServiceResponse, Error> {
    if id.id().is_err() {
        // Redirect to a login page or any other desired endpoint
        return Ok(actix_web::dev::ServiceResponse::new(
            req,
            HttpResponse::Ok()
                .insert_header((actix_web::http::header::LOCATION, "/login"))
                .finish(),
        ));
    }
    // Identity exists, continue with the request handling
    let fut = actix_web::dev::ServiceResponse::new(req, HttpResponse::Ok().finish());
    Ok(future::ready(fut).await)
}

async fn clear_temp_db() {
    use entities::prelude::*;
    use sea_orm::*;
    use tokio::time::Duration;
    loop {
        let conn = DbConnection::connect().await;
        println!("Cleaning temporary rows");
        let rows = TempUsers::find()
            .filter(entities::temp_users::server::Column::Time.lt({
                let current_time = chrono::Utc::now();
                current_time - chrono::Duration::minutes(5)
            }))
            .all(&conn)
            .await
            .unwrap();

        rows.into_iter().for_each(|row| {
            let conn = conn.clone();
            leptos::spawn_local(async move {
                row.delete(&conn).await.unwrap();
            })
        });
        tokio::time::sleep(Duration::from_secs(60 * 15)).await;
    }
}

#[get("/upload/{image_path}")]
async fn image_path(path: actix_web::web::Path<String>) -> impl Responder {
    use std::io::Read;
    let path = std::env::current_dir()
        .unwrap()
        .join("upload/".to_string() + &path.to_string());
    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = database_run().await {
        panic!("Database connection panicked with: {}", err);
    }

    use actix_files::Files;
    use actix_web::middleware::{Compress, Logger, NormalizePath};
    use server_function::{
        AssociatedConversation, ConfirmSubscription, ConversationAction, FindImage,
        GetConversations, GetUsers, HandleMessageInput, HandleSeen, Login, LoginStatus, Logout,
        Redirect, SignUp, Validate, ValidateConversation, VerifyEmail, ViewMessages,
    };
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    use actix_identity::IdentityMiddleware;
    use actix_session::{storage::RedisSessionStore, SessionMiddleware};
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;

    SignUp::register().unwrap();
    Validate::register().unwrap();
    VerifyEmail::register().unwrap();
    ConfirmSubscription::register().unwrap();
    Login::register().unwrap();
    Logout::register().unwrap();
    LoginStatus::register().unwrap();
    Redirect::register().unwrap();
    GetUsers::register().unwrap();
    GetConversations::register().unwrap();
    ConversationAction::register().unwrap();
    ViewMessages::register().unwrap();
    ValidateConversation::register().unwrap();
    AssociatedConversation::register().unwrap();
    HandleMessageInput::register().unwrap();
    HandleSeen::register().unwrap();
    FindImage::register().unwrap();

    tokio::task::spawn_local(clear_temp_db());

    match tokio::process::Command::new("redis-server").spawn().is_ok() {
        true => {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("Started Redis.")
        }
        false => panic!("Failed to start Redist Server"),
    };

    println!("Connecting to database");
    let db_conn = actix_web::web::Data::new(tokio::sync::Mutex::new(DbConnection {
        connection: DbConnection::connect().await,
    }));
    let redis_address = "redis://127.0.0.1:6379";
    let secret_key = Key::generate();
    let redis_store = RedisSessionStore::new(redis_address).await.unwrap();

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let routes = generate_route_list(|cx| view! { cx, <App/> });

        App::new()
            .app_data(db_conn.clone())
            .app_data(actix_web::web::PayloadConfig::new(10_485_760))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(leptos_options.to_owned(), routes, |cx| view! { cx, <App/> })
            .wrap(Logger::new("%r %U").log_target("actix"))
            .wrap(Compress::default())
            .wrap(NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .service(image_path)
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await
    .unwrap();

    Ok(())
}
