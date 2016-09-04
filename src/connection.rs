// setup the uses
// we need the std Read and Write, because otherwise we can't use the read or write
// functions on the TcpStream
use std::io::{Read, Write};
// then we need to load everything from the futures
use futures::{Future, Poll, Async};
// and we need the TcpStream, because otherwise we can't use the type
use tokio_core::{TcpStream};

// then we define our struct, that represents a connection. This will be used in the main.rs
pub struct Connection {
    stream: TcpStream
}

// then we implement some methods for the connection
impl Connection {
    // first we define the new method
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: stream
        }
    }

    // then we define another method, that actually does reading from the socket
    fn do_read(&mut self) -> Result<Vec<u8>, ()> {
        // first we define our buffers. The complete_buffer is the complete message
        // that we receive
        let mut complete_buffer: Vec<u8> = Vec::new();
        // the tmp_buffer has a certain size and will be used for transfering the data
        // over to the complete_buffer
        let mut tmp_buffer = vec![0; 1024];

        // then we loop as long as there is something to read
        loop {
            // we actually read something
            match self.stream.read(&mut tmp_buffer) {
                Ok(len) => {
                    // first we copy the tmp_buffer to the complete_buffer
                    for i in 0..len {
                        complete_buffer.push(tmp_buffer[i]);
                    }

                    // and check if the buffer was full. If it wasn't full, we don't need to read anymore
                    if len < 1024 {
                        break;
                    }
                },
                // if anything went wrong, we ignore it and simply return the complete_buffer
                Err(_) => {
                    return Ok(complete_buffer);
                },
            };
        }
        // now our complete_buffer is filled up with everything read
        // we check whether the complete_buffer has a length equal to 0
        if complete_buffer.len() == 0 {
            // and if it has, we return an error. The connection got broken (closed?)
            return Err(());
        }
        // else we read something
        else {
            // so we return the resulting buffer
            return Ok(complete_buffer);
        }
    }
}

// and we implement everything necessary for the Future trait, so we can actually execute this
impl Future for Connection {
    // now we define the types for this Future. We need to set an empty type, so we can pin.spawn it
    type Item = ();
    type Error = ();

    // and we define the poll method for this
    fn poll(&mut self) -> Poll<(), ()> {
        // then we loop as long as there are some messages to read. This will at least run
        // two times, so the poll_read will setup the next read. We need to execute the poll_read
        // to regiser our interest in the read
        loop {
            // and now we match the result of poll_read, so we know can handle the current situation
            match self.stream.poll_read() {
                // if we can actually read something
                Ok(Async::Ready(_)) => {
                    // we execute do_read and handle it result
                    match self.do_read() {
                        Ok(buf) => {
                            if buf.len() > 0 {
                                self.stream.write_all(&buf).unwrap();
                            }
                        },
                        // if do_read gave an error back, the socket got closed, so finish this future
                        Err(_) => return Ok(Async::Ready(())),
                    }
                },
                // if it's not ready break the loop
                Ok(Async::NotReady) => break,
                // or it an error occured, ignore it and break the loop
                Err(_) => break,
            };
        }

        // and if we reach this, we have registered another interest in reading, so we wait
        return Ok(Async::NotReady);
    }
}