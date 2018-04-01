extern crate actix;
extern crate actix_web;
extern crate openssl;

use actix_web::{Application, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web::StatusCode;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

fn index(_req: HttpRequest) -> Result<HttpResponse, Error> {
    let html = format!(r#"<!DOCTYPE html><html><head><title>thesogu</title></head><body><h1>its soooo guuuu</h1></body></html>"#);

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(&html)?)
}

fn redirect_to_https(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Found()
        .header("LOCATION", "https://thesogu.com")
        .finish().unwrap()
}

fn main() {
    let sys = actix::System::new("thesogu");
    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    ssl_builder
        .set_private_key_file(
            "/etc/letsencrypt/live/www.thesogu.com/privkey.pem",
            SslFiletype::PEM,
        )
        .unwrap();
    ssl_builder
        .set_certificate_chain_file("/etc/letsencrypt/live/www.thesogu.com/fullchain.pem")
        .unwrap();

    let _ = HttpServer::new(|| Application::new().resource("/", |r| r.f(index)))
        .bind("0.0.0.0:443")
        .expect("Cannot bind to 443")
        .start_ssl(ssl_builder)
        .unwrap();

    let _ = HttpServer::new(|| Application::new().resource("/", |r| r.f(redirect_to_https)))
        .bind("0.0.0.0:80")
        .expect("Cannot bind to 80")
        .start();

    let _ = sys.run();
}
