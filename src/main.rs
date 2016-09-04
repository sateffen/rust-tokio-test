#[macro_use]
extern crate tokio_core;
extern crate tokio_proto;
extern crate futures;

use std::io::{Read, Write};
use futures::{Future, Poll, Async};
use futures::stream::Stream;
use tokio_core::{Loop, TcpStream};

struct Connection {
    stream: TcpStream
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        println!("Got Connection");

        Connection {
            stream: stream
        }
    }

    fn do_read(&mut self) -> Result<Vec<u8>, ()> {
        let mut complete_buffer: Vec<u8> = Vec::new();
        let mut tmp_buffer = vec![0; 1024];

        loop {
            match self.stream.read(&mut tmp_buffer) {
                Ok(len) => {
                    for i in 0..len {
                        complete_buffer.push(tmp_buffer[i]);
                    }

                    if len < 1024 {
                        break;
                    }
                },
                Err(_) => {
                    return Ok(complete_buffer);
                },
            };
        }

        if complete_buffer.len() == 0 {
            return Err(());
        }
        else {
            return Ok(complete_buffer);
        }
    }
}

impl Future for Connection {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        loop {
            match self.stream.poll_read() {
                Ok(ready_state) => {
                    match ready_state {
                        Async::Ready(_) => {
                            match self.do_read() {
                                Ok(buf) => {
                                    if buf.len() > 0 {
                                        self.stream.write_all(&buf).unwrap();
                                    }
                                },
                                Err(_) => return Ok(Async::Ready(())),
                            }
                        },
                        Async::NotReady => break,
                    };
                }
                Err(_) => break,
            };
        }

        return Ok(Async::NotReady);
    }
}

pub fn main() {
    // first setup the eventloop
    let mut event_loop = Loop::new().unwrap();

    // setup a tcp listener, that listens for any new connections
    let address = "0.0.0.0:8888".parse().unwrap();
    let tcp_listener = event_loop.handle().tcp_listen(&address);
    let pin = event_loop.pin();

    let server = tcp_listener.and_then(|listener| {
        listener
            .incoming()
            .for_each(|(socket, _)| {
                pin.spawn(Connection::new(socket));

                Ok(())
            })
    });

    event_loop.run(server).unwrap();
}