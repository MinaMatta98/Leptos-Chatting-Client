#![feature(let_chains)]
#![feature(stmt_expr_attributes)]
#![feature(async_closure)]
use actix::Addr;
use actix::*;
use actix_web::web::{self, BytesMut};
use actix_web::{
    cookie::Key, dev, get, http::StatusCode, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
pub use sea_orm::{Database, DbErr, *};
pub mod app;
pub mod database;
pub mod emailing;
pub mod entities;
pub mod migrator;
pub mod server_function;
pub mod web_socket;
use app::*;
use database::{database_run, DbConnection};

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

fn render_images(folder: &str, path: actix_web::web::Path<String>) -> Vec<u8> {
    use std::io::Read;

    let path = std::env::current_dir()
        .unwrap()
        .join(folder.to_string() + "/" + &path);

    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

#[get("/upload/{image_path}")]
async fn image_path(path: actix_web::web::Path<String>, req: HttpRequest) -> HttpResponse {
    let buffer = render_images("upload", path);
    HttpResponse::Ok().body(buffer).respond_to(&req)
}

#[get("/images/{image_path}")]
async fn upload_path(path: actix_web::web::Path<String>, req: HttpRequest) -> HttpResponse {
    let buffer = render_images("images", path);
    HttpResponse::Ok().body(buffer).respond_to(&req)
}

// Entry point for our websocket route
#[get("/ws/{id}")]
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<usize>,
    srv: web::Data<Addr<web_socket::server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        web_socket::session::WsChatSession {
            id: 0,
            hb: std::time::Instant::now(),
            room: *path,
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
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
        AssociatedConversation, ConfirmSubscription, ConversationAction, CreateGroupConversation,
        DeleteConversation, FindImage, GetConversations, GetIcon, GetUser, GetUsers,
        HandleMessageInput, HandleSeen, Login, LoginStatus, Logout, Redirect, SignUp, UploadImage,
        Validate, ValidateConversation, VerifyEmail, ViewMessages, GetImage
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
    GetUser::register().unwrap();
    GetConversations::register().unwrap();
    ConversationAction::register().unwrap();
    ViewMessages::register().unwrap();
    ValidateConversation::register().unwrap();
    AssociatedConversation::register().unwrap();
    HandleMessageInput::register().unwrap();
    HandleSeen::register().unwrap();
    FindImage::register().unwrap();
    DeleteConversation::register().unwrap();
    UploadImage::register().unwrap();
    GetIcon::register().unwrap();
    GetImage::register().unwrap();
    CreateGroupConversation::register().unwrap();

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
    let app_state = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = web_socket::server::ChatServer::new(app_state.clone()).start();
    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let routes = generate_route_list(|cx| view! { cx, <App/> });

        App::new()
            .app_data(db_conn.clone())
            .app_data(actix_web::web::PayloadConfig::new(10_485_760))
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .service(chat_route)
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(leptos_options.to_owned(), routes, |cx| view! { cx, <App/> })
            .wrap(Logger::new("%r %U").log_target("actix"))
            .wrap(Compress::default())
            .wrap(NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .service(image_path)
            .service(upload_path)
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await
    .unwrap();

    Ok(())
}
