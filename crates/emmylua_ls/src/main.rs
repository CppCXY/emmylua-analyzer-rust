mod context;
mod handlers;
mod logger;
mod util;

use handlers::{
    initialized_handler, on_notification_handler, on_req_handler, on_response_handler,
    server_capabilities,
};
use lsp_server::{Connection, Message};
use lsp_types::InitializeParams;
use std::error::Error;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args: Vec<String> = std::env::args().collect();
    let (connection, threads) = if args.len() > 1 {
        let port = args[1].parse::<u16>().unwrap();
        let addr = ("127.0.0.1", port);
        Connection::listen(addr).unwrap()
    } else {
        Connection::stdio()
    };

    let (id, params) = connection.initialize_start()?;
    let initialization_params: InitializeParams = serde_json::from_value(params).unwrap();
    let server_capbilities = server_capabilities();
    let initialize_data = serde_json::json!({
        "capabilities": server_capbilities,
        "serverInfo": {
            "name": CRATE_NAME,
            "version": CRATE_VERSION
        }
    });

    connection.initialize_finish(id, initialize_data)?;

    main_loop(&connection, initialization_params).await?;
    threads.join()?;

    eprintln!("Server shutting down.");
    Ok(())
}

async fn main_loop(
    connection: &Connection,
    params: InitializeParams,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut server_context = context::ServerContext::new(Connection {
        sender: connection.sender.clone(),
        receiver: connection.receiver.clone(),
    });

    let server_context_snapshot = server_context.snapshot();
    tokio::spawn(async move {
        initialized_handler(server_context_snapshot, params).await;
    });

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                on_req_handler(req, &mut server_context).await?;
            }
            Message::Notification(notify) => {
                on_notification_handler(notify, &mut server_context).await?;
            }
            Message::Response(response) => {
                on_response_handler(response, &mut server_context).await?;
            }
        }
    }

    Ok(())
}
