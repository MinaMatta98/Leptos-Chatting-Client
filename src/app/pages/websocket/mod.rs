use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use leptos::html::Input;
use leptos::{
    spawn_local, use_context, NodeRef, RwSignal, Scope, SignalGet, SignalGetUntracked, SignalUpdate,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::app::SeenContext;
use crate::server_function::handle_message_input;

use super::components::avatar::{self, StreamData, WsData, SINKVEC, STREAMVEC};
use super::{Message, UserContext};
use crate::app::pages::components::avatar::ToStreamData;

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

        let _ = SINKVEC::sync_stream(id, data).0.broadcast(message).await;
    }

    pub async fn handle_split_stream<'a, T, E>(
        cx: Scope,
        id: i32,
        messages: Option<RwSignal<T>>,
        url: &str,
        function: impl Fn(Option<&mut T>, E) + 'static,
    ) where
        E: for<'de> Deserialize<'de> + std::any::Any + std::fmt::Debug, // Add this line
        T: std::fmt::Debug,
    {
        let data = match std::any::TypeId::of::<E>() {
            t if t == std::any::TypeId::of::<avatar::IconData>() => WsData::IconData,
            _ => WsData::MessageData,
        };

        let (tx, mut rx, state) = STREAMVEC::sync_stream(id, data);
        let (_, mut rx_sink) = SINKVEC::sync_stream(id, data);

        let messages = move || messages;
        match state {
            avatar::WebSocketState::NewConnection => {
                let (mut sink, mut ws_read) = Self::handle_websocket(url, id);
                spawn_local(async move {
                    while let Some(message) = rx_sink.next().await {
                        sink.send(gloo_net::websocket::Message::Text(
                            serde_json::to_string(&message.into_inner()).unwrap(),
                        ))
                        .await
                        .unwrap();
                    }
                    leptos::on_cleanup(cx, move || {
                        spawn_local(async move {
                            STREAMVEC
                                .write()
                                .retain(|(_, stream_id), _| id != *stream_id);
                            SINKVEC.write().retain(|(_, sink_id), _| id != *sink_id);
                            sink.close().await.unwrap();
                        })
                    });
                });

                spawn_local(async move {
                    while let Some(value) = ws_read.next().await {
                        let result = match value {
                            Ok(gloo_net::websocket::Message::Text(text)) => {
                                let _ = tx
                                    .broadcast(
                                        std::string::String::from_inner(text.trim()).unwrap(),
                                    )
                                    .await;
                                Some(serde_json::from_str(&text))
                            }
                            Ok(gloo_net::websocket::Message::Bytes(bytes)) => {
                                let text = String::from_utf8(bytes).unwrap();
                                Some(serde_json::from_str(&text))
                            }
                            _ => None,
                        };

                        if let Some(Ok(value)) = result {
                            let value: E = value;
                            match messages() {
                                Some(messages) => messages.update(|signal_inner| {
                                    function(Some(signal_inner), value);
                                }),
                                None => function(None, value),
                            }
                        }
                    }
                });
            }
            avatar::WebSocketState::PassThrough => spawn_local(async move {
                while let Some(data) = rx.next().await {
                    let mut value: E;
                    match std::any::TypeId::of::<E>() {
                        t if t == std::any::TypeId::of::<avatar::IconData>() => {
                            value = serde_json::from_value(data.into_inner()).unwrap();
                            value = *Box::<dyn Any>::downcast::<E>(Box::new(value)).unwrap();
                        }
                        _ => {
                            value = serde_json::from_value(data.into_inner()).unwrap();
                            value = *Box::<dyn Any>::downcast::<E>(Box::new(value)).unwrap();
                        }
                    }
                    match messages() {
                        Some(messages) => messages.update(|signal_inner| {
                            function(Some(signal_inner), value);
                        }),
                        None => function(None, value),
                    }
                }
            }),
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
        let body = input_ref.get().unwrap().value();
        let user_context = use_context::<UserContext>(cx).unwrap();
        let seen_context = move || use_context::<SeenContext>(cx).unwrap().status.get();

        if let Some(files) = image_ref.get().unwrap().files() {
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
                        // "ws://localhost:8000/ws/",
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
                    // "ws://localhost:8000/ws/",
                )
                .await
            }
        }
    }
}