use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use leptos::html::Input;
use leptos::{
    spawn_local, use_context, NodeRef, RwSignal, Scope, SignalGet, SignalGetUntracked, SignalSet,
    SignalUpdate,
};
use serde::{Deserialize, Serialize};

use crate::app::SeenContext;
use crate::server_function::handle_message_input;

use super::{Message, UserContext};

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

    pub async fn handle_sink_stream<T: Serialize>(message: T, id: i32, url: &str) {
        let (mut ws_write, _) = Self::handle_websocket(url, id);
        ws_write
            .send(gloo_net::websocket::Message::Text(
                serde_json::to_string(&message).unwrap(),
            ))
            .await
            .unwrap();
    }

    pub async fn handle_split_stream<'a, T, E>(
        cx: Scope,
        id: i32,
        messages: Option<RwSignal<T>>,
        url: &str,
        function: impl Fn(Option<&mut T>, E),
    ) where
        E: for<'de> Deserialize<'de>, // Add this line
    {
        let (mut sink, mut ws_read) = Self::handle_websocket(url, id);
        while let Some(value) = ws_read.next().await {
            let result = match value {
                Ok(gloo_net::websocket::Message::Text(text)) => Some(serde_json::from_str(&text)),
                Ok(gloo_net::websocket::Message::Bytes(bytes)) => {
                    let text = String::from_utf8(bytes).unwrap();
                    Some(serde_json::from_str(&text))
                }
                _ => None,
            };
            // let last = move || messages.get().last().unwrap().message_id + 1;
            if let Some(Ok(value)) = result {
                let value: E = value;
                match messages {
                    Some(messages) => messages.update(|signal_inner| {
                        function(Some(signal_inner), value);
                    }),
                    None => function(None, value),
                }
            }
        }

        leptos::on_cleanup(cx, move || {
            spawn_local(async move {
                sink.close().await.unwrap();
            })
        });
    }

    // pub async fn render_stream_to_signal<T>(
    //     cx: Scope,
    //     signal: RwSignal<T>,
    //     ws_id: i32,
    //     function: impl Fn(String) -> (T, T),
    // ) where
    //     T: Clone,
    // {
    //     let (mut sink, mut stream) =
    //         HandleWebSocket::handle_websocket("ws://localhost:8000/ws/", ws_id);
    //     let recv = move || signal.get();
    //     while let Some(value) = stream.next().await {
    //         signal.set(
    //             match value.unwrap_or_else(|_| gloo_net::websocket::Message::Text("".to_string())) {
    //                 gloo_net::websocket::Message::Text(text) => match text.as_str() {
    //                     "" => recv(),
    //                     _ => function(text).0,
    //                 },
    //                 gloo_net::websocket::Message::Bytes(_) => function(String::new()).1,
    //             },
    //         )
    //     }
    //     leptos::on_cleanup(cx, move || {
    //         spawn_local(async move {
    //             sink.close().await.unwrap();
    //         })
    //     });
    // }
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
                        "ws://localhost:8000/ws/",
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
                    "ws://localhost:8000/ws/",
                )
                .await
            }
        }
    }
}
