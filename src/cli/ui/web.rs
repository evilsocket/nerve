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

    async fn start_websocket_server(&self) -> Result<()> {
        let ws_listener = TcpListener::bind(&self.args.ws_address).await?;

        log::info!("websocket server started on: {}", self.args.ws_address);

        // accept new connections
        let tx = self.events_tx.clone();
        let remote = self.remote.clone();
        tokio::spawn(async move {
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

        Ok(())
    }

    async fn start_http_server(&self) -> Result<()> {
        log::info!("http server started on: http://{}", &self.args.web_address);

        let address = self.args.web_address.clone();
        let body = include_str!("web.html");
        let body = body.replace("{WEBSOCKET_SERVER_ADDRESS}", &self.args.ws_address);

        tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(body.clone()))
                    .route("/", web::get().to(webui_path))
            })
            .bind(&address)
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        Ok(())
    }

    async fn start_control_state_streamer(&self) -> Result<()> {
        let tx = self.events_tx.clone();
        let remote = self.remote.clone();

        tokio::spawn(async move {
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

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        self.start_websocket_server().await?;
        self.start_http_server().await?;
        self.start_control_state_streamer().await?;
        Ok(())
    }
}

pub async fn start(
    events_rx: Receiver,
    events_tx: Sender,
    remote: Control,
    args: Args,
) -> Result<()> {
    let web_ui = WebUI::new(args, events_rx, events_tx, remote).await?;

    web_ui.start().await?;

    Ok(())
}
