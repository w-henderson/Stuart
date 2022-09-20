//! Provides the `stuart dev` functionality.

use crate::build::StuartContext;
use crate::error::StuartError;
use crate::logger::LOGGER;

use humphrey::http::headers::HeaderType;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::{try_find_path, LocatedPath};
use humphrey::stream::Stream;
use humphrey::App;

use humphrey_ws::{Message, WebsocketStream};

use clap::ArgMatches;

use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

/// The WebSocket-based JavaScript to inject into HTML pages, allowing for hot reload.
static JS: &[u8] = include_bytes!("main.js");

/// The state of the Humphrey application used to serve the site.
#[derive(Default)]
struct State {
    /// Connected WebSocket streams to broadcast updates to.
    streams: Arc<Mutex<Vec<WebsocketStream>>>,
}

/// Serves the site with the given arguments.
pub fn serve(args: ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let manifest_path: String = args.value_of("manifest-path").unwrap().to_string();
    let output: String = args.value_of("output").unwrap().to_string();
    let path = PathBuf::try_from(&manifest_path)
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or("invalid manifest path")?;

    let mut ctx = StuartContext::init(&manifest_path, &output, "development")?;

    log!("Started", "development server at http://localhost:6904\n");

    if let Err(e) = ctx.build() {
        error_handler(&e);
    }

    let streams = Arc::new(Mutex::new(Vec::new()));
    let state = State {
        streams: streams.clone(),
    };

    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).unwrap();
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    spawn(move || {
        let app = App::new_with_config(8, state)
            .with_stateless_route("/*", serve_dir)
            .with_websocket_route("/__ws", websocket_handler);

        app.run("127.0.0.1:6904")
            .map_err(|_| Box::new("failed to start development server") as Box<dyn StuartError>)
    });

    build_watcher(rx, streams, path, ctx);

    Ok(())
}

/// Watches for changes to the site, rebuilding and notifying subscribers when necessary.
fn build_watcher(
    rx: Receiver<RawEvent>,
    streams: Arc<Mutex<Vec<WebsocketStream>>>,
    path: PathBuf,
    mut ctx: StuartContext,
) {
    loop {
        if let Ok(e) = rx.recv() {
            let p = e.path.as_ref().unwrap().strip_prefix(&path).unwrap();

            if p.starts_with("dist") || p.starts_with("temp") {
                continue;
            }

            println!();

            if p.ends_with("stuart.toml") {
                log!(
                    "Detected",
                    "configuration change, please restart the server"
                );
                continue;
            }

            log!(
                "Detected",
                "change at {}, rebuilding",
                e.path
                    .unwrap()
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\")
            );

            if let Err(e) = ctx.build() {
                error_handler(&e);
            } else {
                let mut streams = streams.lock().unwrap();
                let mut to_remove = Vec::with_capacity(streams.len());

                #[allow(clippy::significant_drop_in_scrutinee)]
                for (i, stream) in streams.iter_mut().enumerate() {
                    if stream.send(Message::new("reload")).is_err() {
                        to_remove.push(i);
                    }
                }

                for i in to_remove.iter().rev() {
                    streams.swap_remove(*i);
                }
            }

            // TODO: WebSocket stuff

            while rx.try_recv().is_ok() {}
        }
    }
}

/// Handles WebSocket connections to the Humphrey server.
fn websocket_handler(request: Request, stream: Stream, state: Arc<State>) {
    humphrey_ws::websocket_handler(|stream, state: Arc<State>| {
        state.streams.lock().unwrap().push(stream)
    })(request, stream, state);
}

/// Serves a directory.
///
/// Taken from Humphrey ([permalink](https://github.com/w-henderson/Humphrey/blob/8bf07aada8acb7e25991ac9e9f9462d9fb3086b0/humphrey/src/handlers.rs#L78)) and modified to correctly inject the WebSocket code.
fn serve_dir(request: Request) -> Response {
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
                        if let Some(index) = buf.windows(7).position(|w| w == b"</body>") {
                            let mut to_inject = Vec::with_capacity(JS.len() + 17);
                            to_inject.extend_from_slice(b"<script>");
                            to_inject.extend_from_slice(JS);
                            to_inject.extend_from_slice(b"</script>");

                            buf.splice(index..index, to_inject);
                        }

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

/// Prints errors.
#[allow(clippy::borrowed_box)]
fn error_handler(e: &Box<dyn StuartError>) {
    if LOGGER.get().unwrap().has_logged() {
        println!();
    }

    e.print();
}
