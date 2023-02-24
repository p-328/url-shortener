use dotenv::dotenv;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, http::header};
use sqlx::postgres::PgPool;
use sqlx::query;
use serde::{Serialize, Deserialize};
use env_logger;
use url::Url;
use uuid::Uuid;

const HOSTNAME: &str = "127.0.0.1";

#[derive(Serialize)]
struct URLStruct {
    url: String
}

#[derive(Deserialize)]
struct InputURLStruct {
    url_id: Option<String>,
    url: String
}

fn generate_url_safe_id() -> String {
    let mut url_safe_id = String::new();
    let generated_id = Uuid::new_v4().to_string();
    for char in generated_id.chars() {
        if char != '-' {
            url_safe_id.push(char);
        }
    }
    url_safe_id
}

async fn urls(db: web::Data<PgPool>) -> impl Responder {
    let url_shorteners = query!("SELECT * FROM shortened_url;")
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to perform query!");
    let urls: Vec<URLStruct> = url_shorteners.iter().map(|record| -> URLStruct {
        let url_id = record.url_id.as_ref().unwrap();
        URLStruct {
            url: format!("{HOSTNAME}/{url_id}")
        }
    }).collect();  

    HttpResponse::Ok().json(urls)
}


async fn short_link(db: web::Data<PgPool>, params: web::Path<(String,)>) -> impl Responder {
    let (url_id,) = params.into_inner();
    let url_rows = query!("SELECT * FROM shortened_url WHERE url_id = $1 LIMIT 1;", url_id)
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


async fn create_shortened_url(db: web::Data<PgPool>, body: web::Json<InputURLStruct>) -> impl Responder {
    let url = body.0.url;
    if let Err(_) = Url::parse(url.as_str()) {
        return HttpResponse::UnprocessableEntity().body("Invalid URL!");
    }
    if let Some(custom_id) = body.0.url_id {
        let _ = query!("INSERT INTO shortened_url(url_id, url) VALUES ($1, $2);", custom_id, url)
            .execute(db.as_ref())
            .await
            .expect("Failed to perform query!");
        return HttpResponse::Created().json(URLStruct {
            url: format!("{HOSTNAME}/{custom_id}")
        });
    }
    let new_url_id = generate_url_safe_id();
    let _ = query!("INSERT INTO shortened_url(url_id, url) VALUES ($1, $2);", new_url_id, url)
        .execute(db.as_ref())
        .await
        .expect("Failed to perform query!");
    HttpResponse::Created().json(URLStruct {
        url: format!("127.0.0.1:8888/{new_url_id}")
    })
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
            .route("/sl/{url_id}", web::get().to(short_link))
            .route("/create", web::post().to(create_shortened_url))
    })   
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
