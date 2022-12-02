mod request;
mod response;

use clap::Parser;

use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[command(about = "Fun with load balancing")]
struct CmdOptions {
    /// "IP/port to bind to"
    #[arg(short, long, default_value = "0.0.0.0:1100")]
    bind: String,
    /// "Upstream host to forward requests to"
    #[arg(short, long)]
    upstream: Vec<String>,
    /// "Perform active health checks on this interval (in seconds)"
    #[arg(long, default_value = "10")]
    active_health_check_interval: usize,
    /// "Path to send request to for active health checks"
    #[arg(long, default_value = "/")]
    active_health_check_path: String,
    /// "Maximum number of requests to accept per IP per minute (0 = unlimited)"
    #[arg(long, default_value = "0")]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
#[derive(Clone)]
struct ProxyState {
    /// How frequently we check whether upstream servers are alive
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute
    max_requests_per_minute: usize,
    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,
    /// Addresses of servers that are alive
    live_upstream_addresses: Arc<RwLock<Vec<String>>>,
    /// Rate limiting counter
    rate_limiting_counter: Arc<Mutex<HashMap<String, usize>>>,
}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    let state = ProxyState {
        live_upstream_addresses: Arc::new(RwLock::new(options.upstream.clone())),
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        rate_limiting_counter: Arc::new(Mutex::new(HashMap::new())),
    };

    // Start active health check
    let state_temp = state.clone();
    tokio::spawn(async move {
        active_health_check(&state_temp).await;
    });

    // Start cleaning up rate limiting counter every minute
    let state_temp = state.clone();
    tokio::spawn(async move {
        rate_limiting_counter_clearer(&state_temp, 60).await;
    });

    // Handle incoming connections
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let state = state.clone();
            // new tokio task
            tokio::spawn(async move {
                handle_connection(stream, &state).await;
            });
        }
    }
}

async fn rate_limiting_counter_clearer(state: &ProxyState, clear_interval: u64) {
    loop {
        sleep(Duration::from_secs(clear_interval)).await;
        // Clean up counter every minute
        let mut rate_limiting_counter = state.rate_limiting_counter.clone().lock_owned().await;
        rate_limiting_counter.clear();
    }
}

async fn active_health_check(state: &ProxyState) {
    loop {
        sleep(Duration::from_secs(
            state.active_health_check_interval.try_into().unwrap(),
        ))
        .await;

        let mut live_upstream_addresses = state.live_upstream_addresses.write().await;
        live_upstream_addresses.clear();
        // send a request to each upstream
        // If a failed upstream returns HTTP 200, put it back in the rotation of upstream servers.
        // If an online upstream returns a non-200 status code, mark that server as failed.
        for upstream_ip in &state.upstream_addresses {
            let request = http::Request::builder()
                .method(http::Method::GET)
                .uri(&state.active_health_check_path)
                .header("Host", upstream_ip)
                .body(Vec::new())
                .unwrap();
            // Open a connection to a destination server
            match TcpStream::connect(upstream_ip).await {
                Ok(mut conn) => {
                    // Write to stream and read from stream
                    if let Err(error) = request::write_to_stream(&request, &mut conn).await {
                        log::error!(
                            "Failed to send request to upstream {}: {}",
                            upstream_ip,
                            error
                        );
                        return;
                    }
                    let response =
                        match response::read_from_stream(&mut conn, &request.method()).await {
                            Ok(response) => response,
                            Err(error) => {
                                log::error!("Error reading response from server: {:?}", error);
                                return;
                            }
                        };
                    // Handle the statusCode of response
                    match response.status().as_u16() {
                        200 => {
                            live_upstream_addresses.push(upstream_ip.clone());
                        }
                        status @ _ => {
                            log::error!(
                                "upstream server {} is not working: {}",
                                upstream_ip,
                                status
                            );
                            return;
                        }
                    }
                }
                Err(err) => {
                    log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
                    return;
                }
            }
        }
    }
}

async fn connect_to_upstream(state: &ProxyState) -> Result<TcpStream, std::io::Error> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    loop {
        let live_upstream_addresses = state.live_upstream_addresses.read().await;
        let upstream_idx = rng.gen_range(0..live_upstream_addresses.len());
        let upstream_ip = &live_upstream_addresses.get(upstream_idx).unwrap().clone();
        drop(live_upstream_addresses); // release read lock

        match TcpStream::connect(upstream_ip).await {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                // handle dead upstream_addresses
                log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
                let mut live_upstream_addresses = state.live_upstream_addresses.write().await;
                live_upstream_addresses.swap_remove(upstream_idx); // remove the dead upstream

                // All upstreams are dead, return Err
                if live_upstream_addresses.len() == 0 {
                    log::error!("All upstreams are dead");
                    return Err(Error::new(ErrorKind::Other, "All upstreams are dead"));
                }
            }
        }
    }
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!(
        "{} <- {}",
        client_ip,
        response::format_response_line(&response)
    );
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn check_rate(state: &ProxyState, client_conn: &mut TcpStream) -> Result<(), std::io::Error> {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    let mut rate_limiting_counter = state.rate_limiting_counter.clone().lock_owned().await;
    let cnt = rate_limiting_counter.entry(client_ip).or_insert(0);
    *cnt += 1;

    if *cnt > state.max_requests_per_minute {
        let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
        // send_response(&mut client_conn, &response).await;
        if let Err(error) = response::write_to_stream(&response, client_conn).await {
            log::warn!("Failed to send response to client: {}", error);
        }
        return Err(Error::new(ErrorKind::Other, "Rate limiting"));
    }
    Ok(())
}

async fn handle_connection(mut client_conn: TcpStream, state: &ProxyState) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = upstream_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // rate limiting
        if state.max_requests_per_minute > 0 {
            if let Err(_error) = check_rate(&state, &mut client_conn).await {
                log::error!("{} rate limiting", &client_ip);
                continue;
            }
        }

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!(
                "Failed to send request to upstream {}: {}",
                upstream_ip,
                error
            );
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await
        {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
