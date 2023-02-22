use dotenv::dotenv;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use sqlx::{postgres::PgPool};

#[get("/")]
async fn hi(_: web::Data<PgPool>) -> impl Responder {
    HttpResponse::Ok().body("test!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let conn_url = std::env::var("PGURL")
        .expect("PGURL must be set!");
    let pool = web::Data::new(PgPool::connect(conn_url.as_str())
        .await
        .expect("Failed to created database pool"));
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .service(hi)
    })   
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
