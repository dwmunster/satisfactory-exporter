use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "5", help="Interval in seconds between each query to the server")]
    update_interval: u64,

    #[arg(short, long, help="Hostname and port of the server to query")]
    endpoint: String,

    #[arg(short, long, help="File containing the bearer token to use for authentication")]
    token_file: Option<String>,

    #[arg(short, long, help="Allow insecure connections (e.g., to a server with a self-signed certificate)")]
    allow_insecure: bool,

    #[arg(short, long, help="Address:Port to which the server will listen", default_value = "127.0.0.1:3030")]
    listen: String,
}

#[derive(Serialize)]
struct QueryBody {
    function: String,
}

#[derive(Deserialize)]
struct ServerResponse {
    data: ServerData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerData {
    server_game_state: ServerGameState,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerGameState {
    num_connected_players: u64,
    tech_tier: u64,
    total_game_duration: u64,
    average_tick_rate: f64,
}

#[derive(Clone)]
struct Metrics {
    num_connected_players: Gauge,
    tech_tier: Gauge,
    total_game_duration: Gauge,
    average_tick_rate: Gauge,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            num_connected_players: Gauge::new("num_connected_players", "Number of connected players").unwrap(),
            tech_tier: Gauge::new("tech_tier", "Current tech tier").unwrap(),
            total_game_duration: Gauge::new("total_game_duration", "Total game duration").unwrap(),
            average_tick_rate: Gauge::new("average_tick_rate", "Average tick rate").unwrap(),
        }
    }

    fn update(&self, game_state: &ServerGameState) {
        self.num_connected_players.set(game_state.num_connected_players as f64);
        self.tech_tier.set(game_state.tech_tier as f64);
        self.total_game_duration.set(game_state.total_game_duration as f64);
        self.average_tick_rate.set(game_state.average_tick_rate);
    }
}

type SharedState = Arc<(Metrics, Registry)>;

async fn metrics_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&state.1.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let registry = Registry::new();
    let metrics = Arc::new(Metrics::new());

    registry.register(Box::new(metrics.num_connected_players.clone())).unwrap();
    registry.register(Box::new(metrics.tech_tier.clone())).unwrap();
    registry.register(Box::new(metrics.total_game_duration.clone())).unwrap();
    registry.register(Box::new(metrics.average_tick_rate.clone())).unwrap();

    let shared_state: SharedState = Arc::new(((*metrics).clone(), registry));

    let metrics_clone = Arc::clone(&metrics);
    let update_interval = Duration::from_secs(args.update_interval);

    // Build the HTTP client
    let mut client_builder = Client::builder();
    if args.allow_insecure {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let client = client_builder.build()?;

    // Read the bearer token if provided
    let bearer_token = args.token_file.map(|file| fs::read_to_string(file).expect("Failed to read token file"));

    let query_endpoint = format!("https://{}/api/v1", args.endpoint);

    tokio::spawn(async move {
        let mut interval = interval(update_interval);
        let query_body = QueryBody {
            function: "QueryServerState".to_string(),
        };

        loop {
            interval.tick().await;
            let mut request = client.post(&query_endpoint).json(&query_body);

            if let Some(token) = &bearer_token {
                request = request.bearer_auth(token.trim());
            }

            match request.send().await {
                Ok(response) => {
                    match response.json::<ServerResponse>().await {
                        Ok(server_response) => {
                            metrics_clone.update(&server_response.data.server_game_state);
                        }
                        Err(e) => eprintln!("Failed to parse service metrics: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to fetch metrics: {}", e),
            }
        }
    });

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(shared_state);

    let addr = std::net::SocketAddr::from_str(&args.listen)?;
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}