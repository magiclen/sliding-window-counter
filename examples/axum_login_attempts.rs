use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use axum::{
    Router,
    extract::{FromRef, Query, State},
    routing::post,
};
use axum_client_addr::{ClientIp, ClientIpConfig};
use sliding_window_counter::SlidingWindowCounter;

const LOGIN_WINDOW: Duration = Duration::from_secs(60);
const MAX_CLIENT_IPS: u64 = 10000;
const MAX_ATTEMPTS_PER_IP: usize = 5;

#[derive(Clone)]
struct AppState {
    attempts:         SlidingWindowCounter<IpAddr>,
    client_ip_config: ClientIpConfig,
}

impl FromRef<AppState> for ClientIpConfig {
    fn from_ref(state: &AppState) -> Self {
        state.client_ip_config.clone()
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        attempts:         SlidingWindowCounter::new(
            LOGIN_WINDOW,
            MAX_CLIENT_IPS,
            MAX_ATTEMPTS_PER_IP,
        ),
        client_ip_config: ClientIpConfig::builder()
            .build()
            .expect("default client IP config should be valid"),
    };

    let app = Router::new().route("/login", post(login)).with_state(state);
    let listener =
        tokio::net::TcpListener::bind("127.0.0.1:3000").await.expect("listener should bind");

    println!("listening on http://127.0.0.1:3000");
    println!("try: curl -X POST 'http://127.0.0.1:3000/login?password=secret'");
    println!("try: curl -X POST 'http://127.0.0.1:3000/login?password=wrong'");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("server should run");
}

async fn login(
    State(state): State<AppState>,
    client_ip: ClientIp,
    Query(query): Query<HashMap<String, String>>,
) -> String {
    let ip = client_ip.ip();

    // Record before checking the password so both successful and failed attempts are counted.
    let attempts_in_window = state
        .attempts
        .record(ip)
        .map_or_else(|| format!("{MAX_ATTEMPTS_PER_IP}+"), |count| count.to_string());
    let login_ok = query.get("password").is_some_and(|password| password == "secret");
    let result = if login_ok { "success" } else { "failure" };

    format!("ip={ip}\nlogin_result={result}\nattempts_in_last_60_seconds={attempts_in_window}\n")
}
