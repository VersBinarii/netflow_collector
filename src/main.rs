extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
extern crate time;
extern crate netflow_v9;
extern crate clap;
extern crate syslog;

mod writer;
mod file_writer;
mod log;

use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use clap::{Arg, App};


use futures::{Future, Poll, Async};
use futures::{Stream};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use netflow_v9::{Parser};

use writer::Writer;
use file_writer::FileWriter;
use log::Log;

// Needed for CpuPool::executor() if needs be
unsafe impl Send for JSONWriter {}
unsafe impl Sync for JSONWriter {}

struct JSONWriter {
    writer: Box<Writer>,
    // It will recieve the json with data
    rx: Receiver<String>
}

// Since we're using the 
impl Future for JSONWriter {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        //let timeout = Duration::new(5, 0);
        loop {
            match self.rx.recv() {
                Ok(x) => {
                    self.writer.append(&x);
                },
                Err(_) => {
                    return Ok(Async::NotReady)
                }
            }
        }
    }
}

struct NFCollector;

impl UdpCodec for NFCollector {
    type In = (SocketAddr, Vec<u8>);
    type Out = (SocketAddr, Vec<u8>);
    
    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        into.extend(buf);
        addr
    }
}

fn main() {

    let matches = App::new("Netflow colector")
        .version("0.1.0")
        .author("\"Kris\" kris@catdamnit.com")
        .about("Netflow collector")
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .takes_value(true)
             .help("Name of the file the JSON output is stored in. Will be created if does not exists.")
        )
        .arg(Arg::with_name("bind_address")
             .short("b")
             .long("bind")
             .takes_value(true)
             .help("Address to bind. Default: 127.0.0.1")
        )
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .takes_value(true)
             .help("Listening port. Default: 2055")
        )
        .get_matches();

    let log = Log::new();
    log.info(&format!("Starting the Netflow Collector"));

    let out_file = matches.value_of("output").unwrap_or("output.json");
    let address = matches.value_of("bind_address").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("2055");
    let address_and_port = {
        let mut address_and_port = String::from(address);
        address_and_port.push(':');
        address_and_port.push_str(&port);
        address_and_port
    };


    
    let writer = FileWriter::new(Path::new(out_file)).map_err(|err| {
        log.info(&format!("Failed to open {}: {}", out_file, err));
        std::process::exit(1);
    }).unwrap();
    
    let addr: SocketAddr = address_and_port.parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let (tx, rx) = mpsc::channel();
    
    let mut cc = Parser::new();
    
    // Bind both our sockets and then figure out what ports we got.
    let collector = UdpSocket::bind(&addr, &handle).map_err(|_| {
        log.info(&format!("Failed to bind to {}. Exiting...",addr));
        std::process::exit(1);
    }).unwrap();
    
    log.info(&format!("Connected to {}", addr));
    let (_, stream) = collector.framed(NFCollector).split();

    let stream = stream.for_each(move |(addr, message)| {
        
        match cc.parse_netflow_packet(&message, &addr) {   
            Ok(sets) => {
                for s in sets {
                    let x = s.to_json();
                    if let Err(err) = tx.send(x) {
                        log.info(&format!("Error sending over tx channel: {}", err));
                    }
                }
            }
            Err(e) => log.info(&format!("Error parsing netflow packet: {}", e))
        };

        Ok(())
    });
    let mut j_writer = JSONWriter{writer: Box::new(writer), rx: rx};

    std::thread::spawn(move || {
        j_writer.poll().unwrap();
    });

    drop(core.run(stream));
}
