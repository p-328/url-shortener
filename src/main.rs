use dotenv::dotenv;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, http::header};
use sqlx::{postgres::PgPool};
use serde::{Serialize, Deserialize};
use env_logger;

#[derive(Serialize, Deserialize)]
struct URLStruct {
    url: String
}


async fn urls(db: web::Data<PgPool>) -> impl Responder {
    let url_shorteners = sqlx::query!("SELECT * FROM shortened_url;")
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to perform query!");
    let urls: Vec<URLStruct> = url_shorteners.iter().map(|record| -> URLStruct {
        let url_id = record.url_id.as_ref().unwrap();
        URLStruct {
            url: format!("127.0.0.1:8888/{}", url_id)
        }
    }).collect();  

    HttpResponse::Ok().json(urls)
}


async fn short_link(db: web::Data<PgPool>, params: web::Path<(String,)>) -> impl Responder {
    let (url_id,) = params.into_inner();
    let url_rows = sqlx::query!("SELECT * FROM shortened_url WHERE url_id = $1 LIMIT 1;", url_id)
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to perform query!");
    if url_rows.len() == 0 {
        return HttpResponse::NotFound().body("This shortened url does not exist.");
    }
    let identified_url = url_rows.get(0).unwrap().url_id.as_ref().unwrap().clone();
    let redirect_url_string = &identified_url[..];
    let mut response = HttpResponse::TemporaryRedirect();
    response.append_header((header::LOCATION, redirect_url_string));
    response.body("Redirecting...")
}

// TODO: Implement creating URLs. 
async fn create_shortened_url(db: web::Data<PgPool>, body: web::Json<URLStruct>) -> impl Responder {

    HttpResponse::Created().body("URL successfully created.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let conn_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set!");
    let pool = web::Data::new(PgPool::connect(conn_url.as_str())
        .await
        .expect("Failed to created database pool"));
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .route("/", web::get().to(urls))
            .route("/redirect/{url_id}", web::get().to(short_link))
    })   
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
