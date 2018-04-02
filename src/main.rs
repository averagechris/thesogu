extern crate actix;
extern crate actix_web;
extern crate openssl;
#[macro_use]
extern crate tera;

use actix_web::{Application, error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result,
                Method};
use actix_web::fs::{NamedFile, StaticFiles};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use std::env;

struct ApplicationState {
    template: tera::Tera,
}

const DEV_HTTPS_PORT: &'static str = "8443";

fn index(req: HttpRequest<ApplicationState>) -> Result<HttpResponse, error::Error> {
    let mut context = tera::Context::new();
    context.add(
        "sogu_things",
        &vec![
            "sogu",
            "tacos",
            "hating evil corporations",
            "drunk catan",
            "tamales",
            "non evil castros",
        ],
    );

    let html = req.state()
        .template
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template Error"))?;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
        .map_err(|_| error::ErrorInternalServerError("Template Error"))
}

fn redirect_to_https(req: HttpRequest) -> HttpResponse {
    let mut host = String::from(req.headers()["HOST"].to_str().unwrap());

    if host.contains(":") {
        // we are in dev mode or there are some shiesty fucks doing funny shit
        let new_host = host.clone();
        let mut host_port: Vec<_> = new_host.split(":").collect();
        host_port[1] = DEV_HTTPS_PORT;
        host.clear();
        host.push_str(&host_port.join(":"));
    }

    let url = format!("https://{}{}", host, req.uri());
    HttpResponse::Found()
        .header("LOCATION", url)
        .finish()
        .unwrap()
}

fn main() {
    let sys = actix::System::new("thesogu");
    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    let mut key_file = String::new();
    let mut cert_file = String::new();
    let mut ipaddr = String::new();
    let mut http_port = String::new();
    let mut https_port = String::new();

    match env::var_os("THESOGU_DEV") {
        Some(_) => {
            println!("Running in development mode");
            key_file.push_str(concat!(env!("CARGO_MANIFEST_DIR"), "/dev_key.pem"));
            cert_file.push_str(concat!(env!("CARGO_MANIFEST_DIR"), "/dev_cert.pem"));
            ipaddr.push_str("127.0.0.1");
            http_port.push_str("8180");
            https_port.push_str(DEV_HTTPS_PORT);
        }
        None => {
            key_file.push_str("/etc/letsencrypt/live/www.thesogu.com/privkey.pem");
            cert_file.push_str("/etc/letsencrypt/live/www.thesogu.com/fullchain.pem");
            ipaddr.push_str("0.0.0.0");
            http_port.push_str("80");
            https_port.push_str("443");
        }
    }

    let http_ipaddr_port = format!("{}:{}", ipaddr, http_port);
    let https_ipaddr_port = format!("{}:{}", ipaddr, https_port);

    ssl_builder
        .set_private_key_file(key_file, SslFiletype::PEM)
        .unwrap();
    ssl_builder.set_certificate_chain_file(cert_file).unwrap();

    let _ = HttpServer::new(|| {
        let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        Application::with_state(ApplicationState { template: tera })
            .handler("/public", StaticFiles::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public"), true))
            .resource("/", |r| r.method(Method::GET).f(index))
    }).bind(&https_ipaddr_port)
        .expect(&format!("Cannot bind to {}", &https_ipaddr_port))
        .start_ssl(ssl_builder)
        .unwrap();

    let _ = HttpServer::new(|| {
        Application::new().default_resource(|r| r.f(redirect_to_https))
    }).bind(&http_ipaddr_port)
        .expect(&format!("Cannot bind to {}", &http_ipaddr_port))
        .start();

    let _ = sys.run();
}
