use std::net::SocketAddr;

use crate::{
    agent::events::{Event, EventType, Receiver, Sender},
    cli::Args,
    Control, ControlState,
};
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Command {
    Play,
    Pause,
    Stop,
}

struct WebUI {
    args: Args,
    events_rx: Receiver,
    events_tx: Sender,
    remote: Control,
}

async fn webui_path(body: web::Data<String>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body.get_ref().to_owned())
}

impl WebUI {
    pub async fn new(
        args: Args,
        events_rx: Receiver,
        events_tx: Sender,
        remote: Control,
    ) -> Result<Self> {
        Ok(Self {
            args,
            events_rx,
            events_tx,
            remote,
        })
    }

    async fn on_connection(
        mut events_rx: Receiver,
        remote: Control,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<()> {
        log::info!("new connection from: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let (mut ws_writer, mut ws_reader) = ws_stream.split();

        // start a task to read events from the websocket and set the control state
        tokio::spawn(async move {
            while let Some(Ok(event)) = ws_reader.next().await {
                if let Message::Text(msg) = event {
                    log::info!("<{}> received message: {}", addr, msg);
                    let command: Command = serde_json::from_str(&msg).unwrap();
                    match command {
                        Command::Play => remote.set_state(ControlState::Running).await,
                        Command::Pause => remote.set_state(ControlState::Paused).await,
                        Command::Stop => remote.set_state(ControlState::Stopped).await,
                    }
                }
            }
        });

        // start streaming agent events and block until the connection is closed
        while let Ok(event) = events_rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            ws_writer.send(Message::Text(msg.into())).await?;
            log::debug!("sent event to {}", addr);
        }

        Ok(())
    }

    async fn start_websocket_server(&self) -> Result<tokio::task::JoinHandle<()>> {
        let ws_listener = TcpListener::bind(&self.args.ws_address).await?;

        log::info!("websocket server started on: {}", self.args.ws_address);

        // accept new connections
        let tx = self.events_tx.clone();
        let remote = self.remote.clone();
        let handle = tokio::spawn(async move {
            while let Ok((stream, addr)) = ws_listener.accept().await {
                // serve each connection in a separate task
                tokio::spawn(Self::on_connection(
                    tx.subscribe(),
                    remote.clone(),
                    stream,
                    addr,
                ));
            }
        });

        Ok(handle)
    }

    async fn start_http_server(&self) -> Result<tokio::task::JoinHandle<()>> {
        log::info!("http server started on: http://{}", &self.args.web_address);

        let address = self.args.web_address.clone();
        let body = include_str!("web.html");
        let body = body.replace("{WEBSOCKET_SERVER_ADDRESS}", &self.args.ws_address);
        let body = body.replace(
            "{TASK_NAME}",
            if let Some(workflow) = &self.args.workflow {
                workflow
            } else {
                self.args.tasklet.as_deref().unwrap_or("task")
            },
        );
        let body = body.replace("{GENERATOR}", &self.args.generator);

        let handle = tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(body.clone()))
                    .route("/", web::get().to(webui_path))
            })
            .disable_signals()
            .bind(&address)
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        Ok(handle)
    }

    async fn start_control_state_streamer(&self) -> Result<tokio::task::JoinHandle<()>> {
        let tx = self.events_tx.clone();
        let remote = self.remote.clone();

        let handle = tokio::spawn(async move {
            loop {
                let state = remote.get_state().await;

                if tx
                    .send(Event::new(EventType::ControlStateChanged(state.clone())))
                    .is_err()
                {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        });

        Ok(handle)
    }

    async fn wait_for_handles<T: Send + 'static>(
        handles: Vec<tokio::task::JoinHandle<T>>,
    ) -> Vec<T> {
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        results
    }

    async fn merge_handles<T: Send + 'static>(
        handles: Vec<tokio::task::JoinHandle<T>>,
    ) -> tokio::task::JoinHandle<Vec<T>> {
        tokio::spawn(Self::wait_for_handles(handles))
    }

    pub async fn start(&self) -> Result<tokio::task::JoinHandle<Vec<()>>> {
        let handles = vec![
            self.start_websocket_server().await?,
            self.start_http_server().await?,
            self.start_control_state_streamer().await?,
        ];

        Ok(Self::merge_handles(handles).await)
    }
}

pub async fn start(
    events_rx: Receiver,
    events_tx: Sender,
    remote: Control,
    args: Args,
) -> Result<tokio::task::JoinHandle<Vec<()>>> {
    let web_ui = WebUI::new(args, events_rx, events_tx, remote).await?;

    web_ui.start().await
}
