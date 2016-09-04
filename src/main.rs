// load all extern crates
extern crate tokio_core;
extern crate tokio_proto;
extern crate futures;

// load all local modules
mod connection;

// and setup all uses
// first we want to use our self-defined connection 
use connection::Connection;
// then we need to load the Future, otherwise we can't use the Future from tcp_listen
use futures::{Future};
// then we need to load the Stream, otherwise we can't use the Stream from incoming 
use futures::stream::Stream;

// here we start
pub fn main() {
    // first setup the eventloop. No eventloop, no fun
    let mut event_loop = tokio_core::Loop::new().unwrap();

    // then we setup the tcp listener, by first parsing it's address
    let address = "0.0.0.0:8888".parse().unwrap();
    // and taking a handle to the loop and create the listener itself (or at least a future
    // that resolves to the listener)
    let tcp_listener = event_loop.handle().tcp_listen(&address);
    // then we create a pinned handle. This pinned handle will spawn our connections in the
    // eventloop, so they get executed
    let pin = event_loop.pin();

    // then we wait til our listener is ready
    let server = tcp_listener.and_then(|listener| {
        // now we have the listener
        listener
            // so we take a stream of incoming sockets
            .incoming()
            // and spawn each socket as connection to the eventloop, via pin.spawn(Future)
            .for_each(|(socket, _)| {
                // so here we do the actual spawn. This will trigger a call to the Connection::poll function
                pin.spawn(Connection::new(socket));

                // we need to return an empty Result, just see the docs for details
                Ok(())
            }) // and, we return the result of for_each from the and_then :)
    });

    // and tell the eventloop to actually execute the listener retrieval
    event_loop.run(server).unwrap();
}