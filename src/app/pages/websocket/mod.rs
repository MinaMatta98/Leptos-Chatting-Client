use async_broadcast::{Receiver, Sender};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{select, FutureExt, SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use leptos::html::Input;
use leptos::{
    log, spawn_local, use_context, NodeRef, RwSignal, Scope, SignalGet, SignalGetUntracked,
    SignalUpdate,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

use super::components::avatar::{self, IconData, SINKVEC, STREAMVEC};
use super::conversation::Message;
use super::UserContext;
use crate::app::{pages::components::avatar::ToStreamData, SeenContext};
use crate::server_function::handle_message_input;

#[derive(Debug, Clone)]
pub enum SyncChannel {
    BroadCast(Sender<StreamData>, Receiver<StreamData>),
    Mpsc(Sender<StreamData>, Receiver<StreamData>),
}

impl SyncChannel {
    pub async fn send(&mut self, message: StreamData) {
        match self {
            SyncChannel::BroadCast(tx, _) => {
                let _ = tx.broadcast(message).await.unwrap();
            }
            SyncChannel::Mpsc(tx, _) => {
                tx.broadcast(message)
                    .await
                    .expect("Failed to Send Message to Other Threads");
            }
        }
    }

    pub async fn next(&mut self) -> Option<StreamData> {
        match self {
            SyncChannel::BroadCast(_, ref mut rx) => rx.next().await,
            SyncChannel::Mpsc(_, ref mut rx) => rx.next().await,
        }
    }

    pub async fn rebound_stream<E, T: 'static>(
        &mut self,
        messages: impl Fn() -> Option<RwSignal<T>>,
        function: impl Fn(Option<&mut T>, E) + 'static,
    ) where
        E: for<'de> Deserialize<'de> + std::any::Any + std::fmt::Debug, // Add this line
    {
        while let Some(data) = self.next().await {
            let mut value: E;
            value = serde_json::from_value(data.into_inner()).unwrap();
            value = *Box::<dyn Any>::downcast::<E>(Box::new(value)).unwrap();
            match messages() {
                Some(messages) => messages.update(|signal_inner| {
                    function(Some(signal_inner), value);
                }),
                None => function(None, value),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamData {
    Message(Message),
    IconData(IconData),
    Close,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WsData {
    IconData,
    MessageData,
}

impl ToStreamData for String {
    fn from_inner(inner: &str) -> Result<StreamData, std::io::Error> {
        let value: serde_json::Value = serde_json::from_str(inner.trim())?;
        if let Ok(message) = serde_json::from_value::<Message>(value.clone()) {
            return Ok(StreamData::Message(message));
        }
        if let Ok(icon_data) = serde_json::from_value::<IconData>(value) {
            Ok(StreamData::IconData(icon_data))
        } else {
            log!("Error with stream text: {}", inner);
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Data",
            ))
        }
    }
}

impl StreamData {
    pub fn into_inner(self) -> serde_json::Value {
        match self {
            Self::Message(message) => serde_json::to_value(message).unwrap(),
            Self::IconData(icon_data) => serde_json::to_value(icon_data).unwrap(),
            Self::Close => serde_json::to_value("command: close").unwrap(),
        }
    }
}
pub struct HandleWebSocket;

impl HandleWebSocket {
    fn handle_websocket(
        url: &str,
        id: i32,
    ) -> (
        SplitSink<WebSocket, gloo_net::websocket::Message>,
        SplitStream<WebSocket>,
    ) {
        let ws = WebSocket::open(&format!("{url}{id}")).unwrap();
        ws.split()
    }

    pub async fn handle_sink_stream<T: Serialize + 'static>(message: T, id: i32) {
        let data: WsData;
        let message = match std::any::TypeId::of::<T>() {
            t if t == std::any::TypeId::of::<avatar::IconData>() => {
                data = WsData::IconData;
                StreamData::IconData(
                    *Box::<dyn Any>::downcast::<avatar::IconData>(Box::new(message)).unwrap(),
                )
            }
            _ => {
                data = WsData::MessageData;
                StreamData::Message(
                    *Box::<dyn Any>::downcast::<Message>(Box::new(message)).unwrap(),
                )
            }
        };
        let _ = SINKVEC::sync_stream(id, data).send(message).await;
    }

    pub async fn handle_split_stream<'a, T, E>(
        cx: Scope,
        id: i32,
        messages: Option<RwSignal<T>>,
        url: &str,
        function: impl Fn(Option<&mut T>, E) + 'static + Clone,
    ) where
        E: for<'de> Deserialize<'de> + std::any::Any + std::fmt::Debug, // Add this line
        T: std::fmt::Debug,
    {
        let data = match std::any::TypeId::of::<E>() {
            t if t == std::any::TypeId::of::<avatar::IconData>() => WsData::IconData,
            _ => WsData::MessageData,
        };

        let (mut sync_channel, state) = STREAMVEC::sync_stream(id, data);
        let mut sync_clone = sync_channel.clone();
        let function_clone = function.clone();
        let mut rx_sink = SINKVEC::sync_stream(id, data);
        let mut rx_sink_clone = rx_sink.clone();
        let messages = move || messages;

        spawn_local(async move {
            sync_clone.rebound_stream(messages, function_clone).await;
        });

        if let avatar::WebSocketState::NewConnection = state {
            let (mut sink, mut ws_read) = Self::handle_websocket(url, id);
            leptos::on_cleanup(cx, move || {
                STREAMVEC
                    .write()
                    .retain(|(_, stream_id), _| id != *stream_id);
                SINKVEC.write().retain(|(_, sink_id), _| id != *sink_id);
                log!("CLEANING WEBSOCKET");
                spawn_local(async move {
                    rx_sink_clone.send(StreamData::Close).await;
                })
            });

            spawn_local(async move {
                loop {
                    select! {
                            message = rx_sink.next().fuse() => {
                                if message.as_ref().unwrap() == &StreamData::Close {
                                    sink.close().await.unwrap()
                                } else {
                                sink.send(gloo_net::websocket::Message::Text(
                                    serde_json::to_string(&message.unwrap().into_inner()).unwrap(),
                                ))
                                .await
                                .unwrap()
                            }
                            },
                            value = ws_read.next().fuse() => {
                                if let Some(value) = value {
                                    match value {
                                    Ok(gloo_net::websocket::Message::Text(text)) => {
                                        let _ = sync_channel
                                            .send(std::string::String::from_inner(text.trim()).unwrap())
                                            .await;
                                        let value: E = serde_json::from_str(&text).unwrap();
                                        match messages() {
                                            Some(messages) => messages.update(|signal_inner| {
                                                function(Some(signal_inner), value);
                                            }),
                                            None => function(None, value),
                                        };
                                    },
                                    Ok(gloo_net::websocket::Message::Bytes(_)) => {
                                        log!("BYTES?");
                                    },
                                    _ => (),
                                };
                            }
                        }
                    }
                }
            });
        }
    }
}

pub struct UserInputHandler;

impl UserInputHandler {
    pub async fn handle_message(
        cx: Scope,
        image_ref: NodeRef<Input>,
        input_ref: NodeRef<Input>,
        id: i32,
    ) {
        let body = input_ref.get_untracked().unwrap().value();
        let user_context = use_context::<UserContext>(cx).unwrap();
        let seen_context = move || use_context::<SeenContext>(cx).unwrap().status.get();

        if let Some(files) = image_ref.get_untracked().unwrap().files() {
            let list = gloo_file::FileList::from(files);
            if let Some(file) = list.first() {
                let file = Some(gloo_file::futures::read_as_bytes(file).await.unwrap());
                if let Ok(path) = handle_message_input(cx, id, None, file.clone()).await {
                    HandleWebSocket::handle_sink_stream(
                        Message {
                            message: None,
                            image: path,
                            conversation_id: id,
                            first_name: user_context.first_name.get_untracked(),
                            last_name: user_context.last_name.get_untracked(),
                            user_id: user_context.id.get_untracked(),
                            seen: None,
                            message_id: {
                                if let Some(message) = seen_context()
                                    .iter()
                                    .find(|conversation| conversation.conversation_id == id)
                                {
                                    message.last_message_id + 1
                                } else {
                                    0
                                }
                            },
                        },
                        id,
                    )
                    .await;
                }
            } else if (handle_message_input(cx, id, Some(body.clone()), None).await).is_ok() {
                HandleWebSocket::handle_sink_stream(
                    Message {
                        message: Some(body),
                        image: None,
                        conversation_id: id,
                        first_name: user_context.first_name.get_untracked(),
                        last_name: user_context.last_name.get_untracked(),
                        user_id: user_context.id.get_untracked(),
                        seen: None,
                        message_id: {
                            if let Some(message) = seen_context()
                                .iter()
                                .find(|conversation| conversation.conversation_id == id)
                            {
                                message.last_message_id + 1
                            } else {
                                0
                            }
                        },
                    },
                    id,
                )
                .await
            }
        }
    }
}
