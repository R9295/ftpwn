
pub use crate::channels::{channel, Receiver, Sender};

use std::{
    io::{Read, Result, Write},
    net::TcpStream,
};

pub const MAX_MESSAGE_SIZE: usize = 512;

pub trait Connect_To_Server<'a> {
    fn new(host: &'a String) -> Self;
    fn send(&self,credential: &str, stream: &mut TcpStream, sender: &mut Sender<u32>) -> Result<u8>;
    fn can_connect(&self,stream: &mut TcpStream) -> bool;
    fn connect_tcp(&self) -> TcpStream;
}


pub struct Server<'a>{
    pub host: &'a String
}

impl<'a> Connect_To_Server<'a> for Server<'a> {
    fn new(host: &'a String) -> Self {
        Server::<'a> {
        host:host}
    }

    fn connect_tcp(&self) -> TcpStream {
        return  TcpStream::connect(self.host.clone()).unwrap();
    }

    fn send(&self,credential: &str, stream: &mut TcpStream, sender: &mut Sender<u32>) -> Result<u8> {
        if let [username, password] = &credential.split(':').take(2).collect::<Vec<&str>>()[..] {
            sender.send(1);
            let mut buffer = [0; MAX_MESSAGE_SIZE];
            println!("attempting: {} {}", username, password);
            stream.write(format!("USER {}\r\n", username).as_bytes())?;
            stream.read(&mut buffer)?;

            // code 331 (User name okay, need password)
            if buffer[..3] == [51, 51, 49] {
                
                stream.write(format!("PASS {}\r\n", password).as_bytes())?;
                stream.read(&mut buffer)?;
                println!("after writing to buffer from send server");

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