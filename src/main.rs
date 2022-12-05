use std::net::SocketAddr;
use std::time::Duration;
use anyhow::Result;
use gumdrop::Options;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::Pod;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::interval;

#[derive(Debug, Options)]
struct Args {
    #[options(default = "1024")]
    bytes: usize,
    #[options(default = "1000")]
    delay: u64,
    #[options(default = "default")]
    ns:    String,
    #[options(default = "1234")]
    port:  u16,
    help:  bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        bytes,
        delay,
        ns,
        port,
        ..
    } = Args::parse_args_default_or_exit();

    let delay = Duration::from_millis(delay);

    tokio::spawn(async move {
        match server(bytes, port).await {
            Ok(()) => println!("server finished"),
            Err(e) => println!("server failed: {e}"),
        }
    });

    tokio::spawn(async move {
        let mut interval = interval(delay);
        loop {
            interval.tick().await;

            match ping(bytes, &ns, port).await {
                Ok(()) => println!("ping finished"),
                Err(e) => println!("ping failed: {e}"),
            }
        }
    });

    let mut sigint  = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv()  => (),
        _ = sigterm.recv() => (),
    };

    Ok(())
}

async fn ping(bytes: usize, ns: &str, port: u16) -> Result<()> {
    let buffer = vec![0; bytes];

    let client = Client::try_default().await?;
    let pods   = Api::<Pod>::namespaced(client, ns);
    let params = Default::default();

    for pod in pods.list(&params).await? {
        let name = pod.metadata.name;
        let addr = pod.status.and_then(|status| status.pod_ip);

        if let Some((name, addr)) = name.zip(addr) {
            let addr = SocketAddr::new(addr.parse()?, port);

            println!("ping {name} -> {addr:?}");

            let mut socket = TcpStream::connect(addr).await?;
            socket.write_all(&buffer).await?;
        }
    }

    Ok(())
}

async fn server(bytes: usize, port: u16) -> Result<()> {
    let host = "0.0.0.0";
    let addr = SocketAddr::new(host.parse()?, port);
    let server = TcpListener::bind(addr).await?;
    loop {
        let (socket, addr) = server.accept().await?;
        println!("connection from {addr:?}");
        tokio::spawn(echo(socket, bytes));
    }
}

async fn echo(mut socket: TcpStream, bytes: usize) -> Result<()> {
    let peer = socket.peer_addr()?;

    let mut buffer = vec![0; bytes];
    socket.read_exact(&mut buffer).await?;
    println!("message from {peer:?}");

    Ok(socket.write_all(&buffer).await?)
}
