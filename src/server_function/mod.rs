#[cfg(feature = "ssr")]
use iter_tools::Itertools;
use leptos::*;
use serde::{Deserialize, Serialize};

pub mod routes;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserModel {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: i64,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ImageAvailability {
    Found,
    Missing,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConversationMeta {
    pub id: i32,
    pub last_message_at: String,
    pub created_at: String,
    pub name: Option<String>,
    pub is_group: i8,
    pub count: usize,
    pub other_users: Vec<(String, String, i32)>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserLogin {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[cfg(feature = "ssr")]
impl From<users::server::Model> for UserModel {
    fn from(value: users::server::Model) -> Self {
        Self {
            id: value.id,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            phone_number: value.phone_number,
            image: value.image,
        }
    }
}

#[cfg(feature = "ssr")]
impl UserLogin {
    fn evaluate_user(user: Option<actix_identity::Identity>) -> Result<UserLogin, ServerFnError> {
        let returned_user: UserLogin;
        Self::server(match &user.unwrap().id() {
            Ok(val) => match serde_json::from_str(val) {
                Ok(val) => {
                    returned_user = val;
                    Ok(returned_user)
                }
                Err(_) => Err(UserValidation::SerializationError),
            },
            Err(_) => Err(UserValidation::NoUser),
        })
    }

    fn server(
        user_evaluation: Result<UserLogin, UserValidation>,
    ) -> Result<UserLogin, ServerFnError> {
        match user_evaluation {
            Ok(val) => Ok(val),
            Err(e) => Err(ServerFnError::Request(format!(
                "User evaluation error: {}",
                serde_json::to_string_pretty(&e).unwrap()
            ))),
        }
    }

    async fn retrieve_user(user: UserLogin, data: &sea_orm::DatabaseConnection) -> UserModel {
        Users::find()
            .filter(users::server::Column::Id.eq(user.id))
            .one(data)
            .await
            .unwrap()
            .map(Into::into)
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FacingMessageInfo {
    pub conversation_id: i32,
    pub user_ids: Vec<i32>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MergedConversation {
    pub conversation_id: i32,
    pub conversation: ConversationInner,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConversationInner {
    pub user_ids: Vec<i32>,
    pub first_name: String,
    pub last_name: String,
    pub is_group: bool,
    pub name: Option<String>,
    pub messages: Vec<MergedMessages>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MergedMessages {
    pub message_conversation_id: i32,
    pub message_id: i32,
    pub message_body: Option<String>,
    pub message_image: Option<String>,
    pub message_sender_id: i32,
    pub seen_status: Vec<SeenMessageFacing>,
    pub created_at: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct SeenMessageFacing {
    pub seen_id: Option<i32>,
    pub message_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct MessageStructFacing {
    pub message_id: i32,
    pub message_body: Option<String>,
    pub message_image: Option<String>,
    pub message_created_at: String,
    pub message_conversation_id: i32,
    pub message_sender_id: i32,
    pub first_name: String,
    pub last_name: String,
}

#[cfg(feature = "ssr")]
use crate::entities::{conversation, user_conversation};

#[derive(Debug, Serialize, Deserialize)]
enum UserValidation {
    NoUser,
    SerializationError,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
    
        use super::entities::prelude::*;
        use super::entities::*;
        use sea_orm::*;
    
        struct RetrieveConversations;
    
        #[derive(Debug, sea_orm::FromQueryResult, Serialize, Clone)]
        struct MessageInfo {
            conversation_id: i32,
            name: Option<String>,
            is_group: bool
        }
    
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct ConversationInfo {
            conversation_id: i32,
            user_ids: i32,
            first_name: String,
            last_name: String,
            email: String
        }
    
        #[derive(Debug, sea_orm::FromQueryResult, Serialize, Clone, PartialEq, Deserialize)]
        pub struct SeenMessageStruct {
           seen_id: Option<i32>,
           message_id: Option<i32>,
           first_name: Option<String>,
           last_name: Option<String>,
        }
    
        impl From<SeenMessageStruct> for SeenMessageFacing {
                fn from(value: SeenMessageStruct) -> Self {
                    Self {
                        seen_id: value.seen_id,
                        message_id: value.message_id,
                        first_name: value.first_name,
                        last_name: value.last_name
                    }
                }
        }
    
        #[derive(Debug, sea_orm::FromQueryResult, Serialize, Clone, PartialEq, Deserialize)]
        pub struct MessageStruct {
            pub message_id: i32,
            pub message_body: Option<String>,
            pub message_image: Option<String>,
            pub message_created_at: sea_orm::prelude::DateTimeUtc,
            pub message_conversation_id: i32,
            pub message_sender_id: i32,
            pub first_name: String,
            pub last_name: String
        }
    
        impl From<MessageStruct> for MessageStructFacing {
                fn from(value: MessageStruct) -> Self {
                    Self {
                        message_id: value.message_id,
                        message_body: value.message_body,
                        message_sender_id: value.message_sender_id,
                        message_image: value.message_image,
                        message_created_at: value.message_created_at.to_string(),
                        message_conversation_id: value.message_conversation_id,
                        first_name: value.first_name,
                        last_name: value.last_name
                    }
                }
            }
    
        impl From<ConversationInfo> for FacingMessageInfo {
                fn from(value: ConversationInfo) -> Self {
                    Self {
                        conversation_id: value.conversation_id,
                        user_ids: vec![value.user_ids],
                        first_name: value.first_name,
                        last_name: value.last_name,
                        email: value.email
                    }
                }
        }
    
    
    impl RetrieveConversations {
    
        async fn retrieve_user_conversations(user: &UserLogin, data: &sea_orm::DatabaseConnection) -> Vec<MessageInfo> {
                    UserConversation::find()
                        .filter(user_conversation::server::Column::UserIds.eq(user.id))
                        .columns::<crate::entities::conversation::server::Column, Vec<_>>(vec![
                            crate::entities::conversation::server::Column::Id,
                            crate::entities::conversation::server::Column::Name,
                            crate::entities::conversation::server::Column::IsGroup,
                        ])
                        .inner_join(Conversation)
                        .into_model::<MessageInfo>()
                        .all(data)
                        .await
                        .unwrap()
        }
    
        async fn retrieve_associated_users(_user: UserLogin, data: &sea_orm::DatabaseConnection, condition: sea_orm::Condition) -> Vec<FacingMessageInfo> {
    
                    let associated_users = UserConversation::find()
                        .filter(condition)
                        .inner_join(Users)
                        .columns::<user_conversation::server::Column, Vec<_>>(vec![
                            user_conversation::server::Column::UserIds,
                            user_conversation::server::Column::ConversationId,
                        ])
                        .columns::<crate::entities::users::server::Column, Vec<_>>(vec![
                            crate::entities::users::server::Column::Id,
                            crate::entities::users::server::Column::FirstName,
                            crate::entities::users::server::Column::LastName,
                            crate::entities::users::server::Column::Email,
                        ])
                        .columns::<crate::entities::conversation::server::Column, Vec<_>>(vec![
                            crate::entities::conversation::server::Column::IsGroup,
                        ])
                        .inner_join(Conversation)
                        .into_model::<ConversationInfo>()
                        .all(data)
                        .await
                        .unwrap();
    
                    associated_users
                        .into_iter()
                        .map_into()
                        .collect()
                }
    
                async fn retrieve_messages(conversations: &Vec<i32>, data: &sea_orm::DatabaseConnection) -> Vec<MessageStructFacing> {
                    let mut condition: Condition = Condition::any();
                    for conversation in conversations {
                        condition = condition.add(message::server::Column::MessageConversationId.eq(*conversation));
                    }
    
                    Message::find().filter(condition).inner_join(Users).columns::<users::server::Column, Vec<_>>(vec![
                            crate::entities::users::server::Column::FirstName,
                            crate::entities::users::server::Column::LastName,
                    ])
                        .order_by_asc(message::server::Column::MessageCreatedAt).into_model::<MessageStruct>().all(data)
                        .await.unwrap().into_iter().map_into().collect()
                }
    
                async fn retrieve_seen(messages: &Vec<MessageStructFacing>, data: &sea_orm::DatabaseConnection) -> Vec<SeenMessageFacing> {
                    use crate::entities::seen_messages;
    
                    let mut condition: Condition = Condition::any();
                    for message in messages {
                        condition = condition.add(seen_messages::server::Column::MessageId.eq(message.message_id));
                    }
    
                    SeenMessages::find().filter(condition)
                        .columns::<crate::entities::users::server::Column, Vec<_>>(vec![
                            crate::entities::users::server::Column::Id,
                            crate::entities::users::server::Column::FirstName,
                            crate::entities::users::server::Column::LastName,
                        ])
                        .join(JoinType::LeftJoin, seen_messages::server::Relation::Users.def()).into_model::<SeenMessageStruct>()
                        .all(data).await.unwrap().into_iter().map_into().collect()
    
                }
    
                async fn retrieve_images(user_id: i32, data: &sea_orm::DatabaseConnection) -> Option<String> {
                    Users::find().filter(users::server::Column::Id.eq(user_id)).one(data).await.unwrap().unwrap().image
                }
    
    }
    
    pub struct AppendDatabase;
    
    impl AppendDatabase {
    
                async fn insert_messages(data: &sea_orm::DatabaseConnection, message_model: crate::entities::message::server::ActiveModel) {
                    let inserted_message = Message::insert(message_model.clone()).exec(data).await.unwrap();
    
                    SeenMessages::insert(seen_messages::server::ActiveModel {
                        message_id: ActiveValue::Set(inserted_message.last_insert_id),
                        seen_id: message_model.message_sender_id
                    }).exec(data).await.unwrap();
    
                }
    
                async fn insert_seen(data: &DatabaseConnection, message_model: Vec<i32>, user_id: i32) {
                    let existing_ids = SeenMessages::find()
                        .filter(seen_messages::server::Column::MessageId.is_in(message_model.clone()))
                        .filter(seen_messages::server::Column::SeenId.eq(user_id))
                        .all(data)
                        .await
                        .unwrap();
    
                    let existing_ids: Vec<i32> = existing_ids.iter().map(|row| row.message_id).collect();
    
                    let new_ids: Vec<i32> = message_model
                        .iter()
                        .filter(|&message_id| !existing_ids.contains(message_id))
                        .copied()
                        .collect();
    
                    if !new_ids.is_empty() {
                        let insert_data: Vec<seen_messages::server::ActiveModel> = new_ids
                            .iter()
                            .map(|&message_id| seen_messages::server::ActiveModel {
                                message_id: ActiveValue::Set(message_id),
                                seen_id: ActiveValue::Set(user_id),
                            })
                            .collect();
                        SeenMessages::insert_many(insert_data)
                            .exec(data)
                            .await
                            .unwrap();
                    }
                }
    
                async fn delete_conversation(conversation_id: i32, data: &sea_orm::DatabaseConnection, user: UserLogin) {
                    if let Ok(conversation) = Conversation::find().
                        filter(Condition::all()
                        .add(conversation::server::Column::Id.eq(conversation_id))
                        .add(user_conversation::server::Column::UserIds.eq(user.id)))
                        .reverse_join(UserConversation)
                        .one(data).await {
                        conversation.unwrap().delete(data).await.unwrap();
                    }
                }
    
                async fn modify(user: UserLogin, image: Option<String>, data: &sea_orm::DatabaseConnection, first_name: Option<String>, last_name: Option<String>) {
                    let mut user_model: users::server::ActiveModel = Users::find_by_id(user.id).one(data).await.unwrap().unwrap().into();
                    if let Some(image_path) = image {
                        user_model.image = Set(Some(image_path));
                    }
    
                    if let Some(first_name) = first_name {
                        user_model.first_name = Set(first_name);
                    }
    
                    if let Some(last_name) = last_name {
                        user_model.last_name = Set(last_name);
                    }
    
                    Users::update(user_model).exec(data).await.unwrap();
                }
    
        }
    }
}
