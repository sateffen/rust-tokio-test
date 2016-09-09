// load all extern crates
extern crate tokio_core;
extern crate futures;

// load all local modules
mod connection;

// and setup all uses
// first we want to use our self-defined connection 
use connection::Connection;
// then we need the stream from futures, because we want to use the incoming
use futures::stream::Stream;
// and we need the tcp listener from tokio core
use tokio_core::net::TcpListener;
// and the actual tokio reactor
use tokio_core::reactor::Core;

// here we start
pub fn main() {
    // first setup the reactor. No reactor, no fun
    let mut reactor = Core::new().unwrap();
    // and we need an actual handle to the reactor, so we can spawn our connections
    let handle = reactor.handle();

    // then we setup the tcp listener, by first parsing it's address
    let address = "0.0.0.0:8888".parse().unwrap();
    // and creating the tcp listener itself. This socket will actually listen for new connections
    let listener = TcpListener::bind(&address, &handle).unwrap();

    // and here we take the new connections We setup a server-future
    let server = listener
        // which is based on listening for each incoming connection
        .incoming()
        // and going for each connection
        .for_each(|(socket, _)| {
            // then spawning our connection itself to the reactor. The connection is a future, so
            // the reactor will actually try to resolve it
            handle.spawn(Connection::new(socket));

            // and finally we return just an Ok, because we have to.
            Ok(())
        });

    // finally we tell the reactor to run our server
    reactor.run(server).unwrap();
}