use std::collections::HashSet;

use actix::{Actor, Addr, AsyncContext, Handler, Message, Recipient, StreamHandler};
use actix_web_actors::ws;
use tokio::sync::mpsc;

#[derive(Message)]
#[rtype(result = "()")]
struct Frame {
    frame: lightfx::Frame,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Connect {
    addr: Recipient<Frame>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    addr: Recipient<Frame>,
}

pub struct FrameBroadcaster {
    receiver: Option<mpsc::Receiver<lightfx::Frame>>,
    recipients: HashSet<Recipient<Frame>>,
}

impl FrameBroadcaster {
    pub fn new(receiver: mpsc::Receiver<lightfx::Frame>) -> Self {
        Self {
            receiver: Some(receiver),
            recipients: HashSet::new(),
        }
    }
}

impl Actor for FrameBroadcaster {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut rx = self.receiver.take().unwrap();
        ctx.add_stream(async_stream::stream! {
            while let Some(item) = rx.recv().await {
                yield item;
            }
        });
    }
}

impl Handler<Connect> for FrameBroadcaster {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.recipients.insert(msg.addr);
    }
}

impl Handler<Disconnect> for FrameBroadcaster {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.recipients.remove(&msg.addr);
    }
}

impl StreamHandler<lightfx::Frame> for FrameBroadcaster {
    fn handle(&mut self, frame: lightfx::Frame, _ctx: &mut Self::Context) {
        for recipient in self.recipients.iter() {
            recipient.do_send(Frame {
                frame: frame.clone(),
            })
        }
    }
}

pub struct FrameBroadcasterSession {
    server: Addr<FrameBroadcaster>,
}

impl FrameBroadcasterSession {
    pub fn new(server: Addr<FrameBroadcaster>) -> Self {
        Self { server }
    }
}

impl Actor for FrameBroadcasterSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(Connect {
            addr: ctx.address().recipient(),
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(Disconnect {
            addr: ctx.address().recipient(),
        })
    }
}

impl Handler<Frame> for FrameBroadcasterSession {
    type Result = ();

    fn handle(&mut self, msg: Frame, ctx: &mut Self::Context) -> Self::Result {
        let bytes: Vec<_> = msg
            .frame
            .pixels_iter()
            .flat_map(|c| [c.r, c.g, c.b])
            .collect();
        ctx.binary(bytes);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FrameBroadcasterSession {
    fn handle(&mut self, _item: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {}
}
