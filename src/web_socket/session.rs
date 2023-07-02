use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use iter_tools::Itertools;
use leptos::log;

use crate::app::pages::components::avatar;
use crate::web_socket::server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    /// unique session id
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// joined room
    pub room: usize,

    /// peer name
    pub name: Option<String>,

    /// Chat server
    pub addr: Addr<server::ChatServer>,
}

pub struct WsChatSessionIcon {
    /// unique session id
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// Chat server
    pub addr: Addr<server::IconWs>,
}

trait Heartbeat<T: Actor> {
    fn hb(&self) -> Instant;
    fn set_hb(&mut self, instant: Instant);
    fn addr(&self) -> &Addr<T>;
    fn id(&self) -> usize;
}

impl Heartbeat<server::ChatServer> for WsChatSession {
    fn hb(&self) -> Instant {
        self.hb
    }

    fn set_hb(&mut self, instant: Instant) {
        self.hb = instant;
    }

    fn addr(&self) -> &Addr<server::ChatServer> {
        &self.addr
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl Heartbeat<server::IconWs> for WsChatSessionIcon {
    fn hb(&self) -> Instant {
        self.hb
    }

    fn set_hb(&mut self, instant: Instant) {
        self.hb = instant;
    }

    fn addr(&self) -> &Addr<server::IconWs> {
        &self.addr
    }

    fn id(&self) -> usize {
        self.id
    }
}

fn handle_hb<T, E>(act: &mut T, ctx: &mut ws::WebsocketContext<T>)
where
    T: Actor<Context = ws::WebsocketContext<T>> + Heartbeat<E>,
    E: Actor + actix::Handler<server::Disconnect> + actix::Handler<server::ClientMessage>,
    <E as actix::Actor>::Context: actix::dev::ToEnvelope<E, server::Disconnect>
        + actix::dev::ToEnvelope<E, server::ClientMessage>,
{
    // check client heartbeats
    if Instant::now().duration_since(act.hb()) > CLIENT_TIMEOUT {
        // heartbeat timed out
        println!("Websocket Client heartbeat failed, disconnecting!");

        // notify chat server
        act.addr().do_send(server::Disconnect { id: act.id() });

        // stop actor
        ctx.stop();

        // don't try to send a ping
        return;
    }

    ctx.ping(b"");
}

impl WsChatSessionIcon {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            handle_hb(act, ctx);
        });
    }
}

impl WsChatSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            handle_hb(act, ctx);
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect(addr.recipient(), self.room))
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

impl Actor for WsChatSessionIcon {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::IconConnect(addr.recipient(), self.id))
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

impl Handler<server::IconWsMessage> for WsChatSessionIcon {
    type Result = ();

    fn handle(&mut self, msg: server::IconWsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let mut text: crate::app::pages::Message =
                    serde_json::from_str(&String::from_utf8(text.into_bytes().to_vec()).unwrap())
                        .unwrap();
                let crate::app::pages::Message {
                    last_name,
                    user_id,
                    first_name,
                    ..
                } = text.clone();
                self.room = text.conversation_id as usize;
                self.addr.do_send(server::Join {
                    id: self.id,
                    name: self.room,
                    conversation_id: self.room,
                    last_name,
                    first_name,
                    user_id: user_id as usize,
                });

                // ctx.text(text.message)
                let room = self.room;
                self.addr
                    .send(server::ListUsers)
                    .into_actor(self)
                    .then(move |res, sess, _ctx| {
                        match res {
                            Ok(users) => {
                                let filtered_users: Vec<_> = users
                                    .iter()
                                    .filter_map(|conversations| {
                                        if conversations.conversation_id == room {
                                            Some((
                                                conversations.first_name.clone(),
                                                conversations.last_name.clone(),
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .sorted()
                                    .unique()
                                    .collect();
                                text.seen = Some(filtered_users);
                                log!("TEXT RECEIVED {:?}", text);
                                log!("ROOM {} ID {}", sess.room, sess.id);
                                sess.addr.do_send(server::ClientMessage {
                                    id: sess.id,
                                    msg: serde_json::to_string_pretty(&text).unwrap(),
                                    room: sess.room,
                                });
                            }
                            _ => println!("Something is wrong"),
                        }
                        fut::ready(())
                    })
                    .wait(ctx);
                // ctx.spawn(future);
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSessionIcon {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let text: avatar::IconData =
                    serde_json::from_str(&String::from_utf8(text.into_bytes().to_vec()).unwrap())
                        .unwrap();
                self.addr.do_send(server::JoinIconWs {
                    id: text.user_id as usize,
                });

                self.addr.do_send(server::ClientMessage {
                    id: text.user_id as usize,
                    msg: serde_json::to_string_pretty(&text).unwrap(),
                    room: text.user_id as usize,
                });
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}
