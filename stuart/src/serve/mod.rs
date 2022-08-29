use crate::error::StuartError;
use crate::logger::LOGGER;

use humphrey::http::headers::HeaderType;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::{try_find_path, LocatedPath};
use humphrey::App;

use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::spawn;

pub fn serve() {
    if let Err(e) = crate::build::build("stuart.toml", "dist") {
        error_handler(&e);
    }

    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).unwrap();
    watcher.watch(".", RecursiveMode::Recursive).unwrap();
    spawn(move || build_watcher(rx));

    let app = App::new_with_config(8, ()).with_route("/*", serve_dir);

    app.run("127.0.0.1:6904").unwrap();
}

fn build_watcher(rx: Receiver<RawEvent>) {
    loop {
        if let Ok(e) = rx.recv() {
            if e.path
                .as_ref()
                .unwrap()
                .components()
                .any(|c| c.as_os_str() == "dist" || c.as_os_str() == "temp")
            {
                continue;
            }

            println!();

            log!(
                "Detected",
                "change at {}, rebuilding",
                e.path.unwrap().display()
            );

            if let Err(e) = crate::build::build("stuart.toml", "dist") {
                error_handler(&e);
            }

            // TODO: WebSocket stuff

            while rx.try_recv().is_ok() {}
        }
    }
}

// Taken from Humphrey and modified to correctly inject the WebSocket code.
// https://github.com/w-henderson/Humphrey/blob/8bf07aada8acb7e25991ac9e9f9462d9fb3086b0/humphrey/src/handlers.rs#L78
fn serve_dir(request: Request, state: Arc<()>) -> Response {
    let uri_without_route = request.uri.strip_prefix('/').unwrap_or(&request.uri);

    let located = try_find_path("dist", uri_without_route, &["index.html"]);

    if let Some(located) = located {
        match located {
            LocatedPath::Directory => Response::empty(StatusCode::MovedPermanently)
                .with_header(HeaderType::Location, format!("{}/", &request.uri)),
            LocatedPath::File(path) => {
                if let Ok(mut file) = File::open(&path) {
                    let mut buf = Vec::new();
                    if file.read_to_end(&mut buf).is_ok() {
                        return if let Some(extension) = path.extension() {
                            Response::new(StatusCode::OK, buf).with_header(
                                HeaderType::ContentType,
                                MimeType::from_extension(extension.to_str().unwrap()).to_string(),
                            )
                        } else {
                            Response::new(StatusCode::OK, buf)
                        };
                    }
                }

                Response::new(StatusCode::InternalError, "Internal Server Error")
            }
        }
    } else {
        Response::new(StatusCode::NotFound, "Not Found")
    }
}

#[allow(clippy::borrowed_box)]
fn error_handler(e: &Box<dyn StuartError>) {
    if LOGGER.get().unwrap().has_logged() {
        println!();
    }

    e.print();
}
