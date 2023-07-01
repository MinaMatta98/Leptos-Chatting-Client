use std::collections::{HashMap, HashSet};

use actix::prelude::*;

use rand::{self, rngs::ThreadRng, Rng};

/// Chat server sends this messages to session
#[derive(Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct Message {
    pub message: String,
    pub conversation_id: usize,
}

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(result = "usize")]
pub struct Connect(pub Recipient<Message>, pub usize);

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: usize,
}

/// List of available rooms
///
pub struct ListUsers;

pub struct IconWsListUsers;
impl actix::Message for IconWsListUsers {
    type Result = Vec<usize>;
}

impl actix::Message for ListUsers {
    type Result = Vec<ConnectedUsers>;
}

pub struct ConnectedUsers {
    pub conversation_id: usize,
    pub user_id: usize,
    pub last_name: String,
    pub first_name: String,
}
// impl actix::Message for ListUsers {
//     type Result = Vec<ListUsers>;
// }

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: usize,
    /// Conversation ID
    pub conversation_id: usize,
    /// Room name
    pub name: usize,
    pub first_name: String,
    pub last_name: String,
    pub user_id: usize,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<usize, HashSet<usize>>,
    rng: ThreadRng,
    // visitor_count: Arc<AtomicUsize>,
    users: HashMap<(usize, usize), (String, String)>,
}

impl Default for ChatServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatServer {
    pub fn new() -> ChatServer {
        // default room
        let rooms = HashMap::new();
        let users = HashMap::new();

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            // visitor_count,
            users,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_message(&self, room: usize, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(&room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message {
                            message: message.to_owned(),
                            conversation_id: room,
                        });
                    }
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.0);

        // auto join session to room with conversation_id
        self.rooms
            .entry(msg.1)
            .or_insert_with(HashSet::new)
            .insert(id);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<usize> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        // for room in rooms {
        //     // self.send_message(room, "Someone disconnected", 0);
        // }
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListUsers> for ChatServer {
    type Result = MessageResult<ListUsers>;

    fn handle(&mut self, _: ListUsers, _: &mut Context<Self>) -> Self::Result {
        let mut users = Vec::new();

        for ((user_id, conversation_id), (first_name, last_name)) in self.users.iter() {
            users.push(ConnectedUsers {
                conversation_id: *conversation_id,
                user_id: *user_id,
                first_name: first_name.clone(),
                last_name: last_name.clone(),
            })
        }

        // MessageResult(users)
        MessageResult(users)
    }
}

impl Handler<IconWsListUsers> for IconWs {
    type Result = MessageResult<IconWsListUsers>;

    fn handle(&mut self, _: IconWsListUsers, _: &mut Context<Self>) -> Self::Result {
        let mut users = Vec::new();

        for user in self.users.iter() {
            users.push(*user)
        }

        // MessageResult(users)
        MessageResult(users)
    }
}
/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {
            id,
            conversation_id,
            user_id,
            first_name,
            last_name,
            ..
        } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        self.users
            .insert((user_id, conversation_id), (first_name, last_name));
        self.rooms
            .entry(conversation_id)
            .or_insert_with(HashSet::new)
            .insert(id);

        // self.send_message(name, "Someone connected", id);
    }
}

#[derive(Debug)]
pub struct IconWs {
    users: Vec<usize>,
    rng: ThreadRng,
    sessions: HashMap<usize, Recipient<IconWsMessage>>,
}

#[derive(Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct IconWsMessage {
    pub message: String,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct IconConnect(pub Recipient<IconWsMessage>, pub usize);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinIconWs {
    /// Client ID
    pub id: usize,
}

impl Handler<JoinIconWs> for IconWs {
    type Result = ();

    fn handle(&mut self, msg: JoinIconWs, _: &mut Context<Self>) {
        let JoinIconWs { id, .. } = msg;

        self.users.push(id);
        // self.send_message(name, "Someone connected", id);
    }
}

impl Handler<IconConnect> for IconWs {
    type Result = usize;

    fn handle(&mut self, msg: IconConnect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        // self.send_message(msg.conversation_id, "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.0);

        // auto join session to room with conversation_id
        // self.users.push(msg.1);

        // send id back
        id
    }
}
impl Actor for IconWs {
    type Context = Context<Self>;
}

impl Default for IconWs {
    fn default() -> Self {
        Self::new()
    }
}

impl IconWs {
    pub fn new() -> IconWs {
        // default room
        let users = Vec::new();
        let sessions = HashMap::new();
        let rng = rand::thread_rng();

        IconWs {
            sessions,
            users,
            rng,
        }
    }
}

impl IconWs {
    /// Send message to all users in the room
    fn send_message(&self, message: &str) {
        for session in self.sessions.iter() {
            session.1.do_send(IconWsMessage {
                message: message.to_string(),
            })
        }
    }
}

impl Handler<ClientMessage> for IconWs {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(msg.msg.as_str());
    }
}

impl Handler<Disconnect> for IconWs {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            self.users = self
                .users
                .iter()
                .filter_map(|&user| if user == msg.id { Some(user) } else { None })
                // .drain_filter(|&mut user| user == msg.id)
                .collect()
        }
    }
}
