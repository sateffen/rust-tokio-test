// setup the uses
// we need the std Read and Write, because otherwise we can't use the read or write
// functions on the TcpStream
use std::io::{Read, Write};
// then we need to load everything from the futures
use futures::{Future, Poll, Async};
// and we need the TcpStream, because otherwise we can't use the type
use tokio_core::net::{TcpStream};

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
            // we actually read something, or attempt to read at least. This gets called while
            // the poll is active, so this implicitly tells tokio to schedule a future read
            match self.stream.read(&mut tmp_buffer) {
                // and if we read something, we collect it to the complete buffer
                Ok(len) => {
                    // if we read less than 1024 byte (which is the size of our tmp_buffer), there
                    // wasn't more to read
                    if len < 1024 {
                        // because we know, that after this the tmp_buffer isn't needed anymore, we
                        // truncate the tmp_buffer to the read length. After this, it's just len long
                        tmp_buffer.truncate(len);
                        // now we extend the complete_buffer by what's left in our tmp_buffer
                        complete_buffer.extend(&tmp_buffer);
                        // and because we don't have anything left to read, we break this loop, so
                        // the followup logic can work with the result
                        break;
                    }
                    // else we read 1024 byte, so we expect even more to read
                    else {
                        // so we simply extend our complete_buffer with our tmp_buffer. We don't need to
                        // truncate tmp_buffer, because we need the complete tmp_buffer
                        complete_buffer.extend(&tmp_buffer);
                    }
                },
                // else an error occured, it's a "WouldBlock" error. That tells us, that there is
                // nothing to read. If there is nothing to read, we return with an Ok, and the complete_buffer
                // the buffer might be 0 length, but that's alright, the next read will come
                Err(_) => return Ok(complete_buffer),
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

    // in this function we handle the actual processing of the received buffer
    fn handle_message(&mut self, buffer: Vec<u8>) {
        // currently we just echo it back, nothing special
        self.stream.write_all(&buffer).unwrap();
    }
}

// and we implement everything necessary for the Future trait, so we can actually execute this
impl Future for Connection {
    // now we define the types for this Future. We need to set an empty type, so we can handle.spawn it
    type Item = ();
    type Error = ();

    // and we define the poll method for this
    fn poll(&mut self) -> Poll<(), ()> {
        // we simply loop as long as we can read
        loop {
            // and check if there is something to read. This will trigger the reactor to schedule something
            // to read in future, just because we execute a read in there.
            match self.do_read() {
                // if we got a buffer we can work with it
                Ok(buf) => {
                    // but first we have to check it's length. If it's longer than 0 bytes, we can work with it
                    if buf.len() > 0 {
                        // by calling handle_message with the buffer
                        self.handle_message(buf);
                    }
                    // else the buffer had a length of 0, so a WouldBlock occured, so there is nothing to read
                    else {
                        // so we simply tell the outer world to check back later on
                        return Ok(Async::NotReady);
                    }
                },
                // else we got an error, that tells us the connection got closed. If it got closed
                // we can tell the future is ready and quit it.
                Err(_) => return Ok(Async::Ready(())),
            };
        }
    }
}