use handler::Handler;
use communication::Sender;

/// A trait for creating new WebSocket handlers.
pub trait Factory {
    type Handler: Handler;

    /// Called when a TCP connection is made.
    fn connection_made(&mut self, _: Sender) -> Self::Handler;

    /// Called when the WebSocket is shutting down.
    #[inline]
    fn on_shutdown(&mut self) {
        debug!("Factory received WebSocket shutdown request.");
    }

    /// Called when a new connection is established for a client endpoint.
    /// This method can be used to differentiate a client aspect for a handler.
    ///
    /// ```
    /// use ws::{Sender, Factory, Handler};
    ///
    /// struct MyHandler {
    ///     ws: Sender,
    ///     is_client: bool,
    /// }
    ///
    /// impl Handler for MyHandler {}
    ///
    /// struct MyFactory;
    ///
    /// impl Factory for MyFactory {
    ///     type Handler = MyHandler;
    ///
    ///     fn connection_made(&mut self, ws: Sender) -> MyHandler {
    ///         MyHandler {
    ///             ws: ws,
    ///             // default to server
    ///             is_client: false,
    ///         }
    ///     }
    ///
    ///     fn client_connected(&mut self, ws: Sender) -> MyHandler {
    ///         MyHandler {
    ///             ws: ws,
    ///             is_client: true,
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn client_connected(&mut self, ws: Sender) -> Self::Handler {
        self.connection_made(ws)
    }

    /// Called when a new connection is established for a server endpoint.
    /// This method can be used to differentiate a server aspect for a handler.
    ///
    /// ```
    /// use ws::{Sender, Factory, Handler};
    ///
    /// struct MyHandler {
    ///     ws: Sender,
    ///     is_server: bool,
    /// }
    ///
    /// impl Handler for MyHandler {}
    ///
    /// struct MyFactory;
    ///
    /// impl Factory for MyFactory {
    ///     type Handler = MyHandler;
    ///
    ///     fn connection_made(&mut self, ws: Sender) -> MyHandler {
    ///         MyHandler {
    ///             ws: ws,
    ///             // default to client
    ///             is_server: false,
    ///         }
    ///     }
    ///
    ///     fn server_connected(&mut self, ws: Sender) -> MyHandler {
    ///         MyHandler {
    ///             ws: ws,
    ///             is_server: true,
    ///         }
    ///     }
    /// }
    #[inline]
    fn server_connected(&mut self, ws: Sender) -> Self::Handler {
        self.connection_made(ws)
    }

    /// Called when a TCP connection is lost with the handler that was
    /// setup for that connection.
    ///
    /// The default implementation is a noop that simply drops the handler.
    /// You can use this to track connections being destroyed or to finalize
    /// state that was not internally tracked by the handler.
    #[inline]
    fn connection_lost(&mut self, _: Self::Handler) {
    }

}

impl<F, H> Factory for F
    where H: Handler, F: FnMut(Sender) -> H
{
    type Handler = H;

    fn connection_made(&mut self, out: Sender) -> H {
        self(out)
    }

}

mod test {
    #![allow(unused_imports, unused_variables, dead_code)]
    use super::*;
    use mio;
    use communication::{Command, Sender};
    use handshake::{Request, Response, Handshake};
    use protocol::CloseCode;
    use frame;
    use message;
    use handler::Handler;
    use result::Result;

    struct S;

    impl mio::Handler for S {
        type Message = Command;
        type Timeout = ();
    }

    #[derive(Debug, Eq, PartialEq)]
    struct M;
    impl Handler for M {
        fn on_message(&mut self, _: message::Message) -> Result<()> {
            Ok(println!("test"))
        }

        fn on_frame(&mut self, f: frame::Frame) -> Result<Option<frame::Frame>> {
            Ok(None)
        }
    }

    #[test]
    fn impl_factory() {

        struct X;

        impl Factory for X {
            type Handler = M;
            fn connection_made(&mut self, _: Sender) -> M {
                M
            }
        }

        let event_loop = mio::EventLoop::<S>::new().unwrap();

        let mut x = X;
        let m = x.connection_made(
            Sender::new(mio::Token(0), event_loop.channel())
        );
        assert_eq!(m, M);
    }

    #[test]
    fn closure_factory() {
        let event_loop = mio::EventLoop::<S>::new().unwrap();

        let mut factory = |_| {
            |_| {Ok(())}
        };

        factory.connection_made(
            Sender::new(mio::Token(0), event_loop.channel())
        );
    }

    #[test]
    fn connection_lost() {
        struct X;

        impl Factory for X {
            type Handler = M;
            fn connection_made(&mut self, _: Sender) -> M {
                M
            }
            fn connection_lost(&mut self, handler: M) {
                assert_eq!(handler, M);
            }
        }

        let event_loop = mio::EventLoop::<S>::new().unwrap();

        let mut x = X;
        let m = x.connection_made(
            Sender::new(mio::Token(0), event_loop.channel())
        );
        x.connection_lost(m);
    }
}
