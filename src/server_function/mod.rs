use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserModel {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: i64,
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
    pub name: String,
    pub is_group: i8,
    pub count: usize,
    pub other_users: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserLogin {
    pub id: i32,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FacingMessageInfo {
    pub conversation_id: Option<i32>,
    pub user_ids: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MergedConversation {
    pub conversation_id: i32,
    pub conversation: ConversationInner,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConversationInner {
    pub user_ids: i32,
    pub first_name: String,
    pub last_name: String,
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

use crate::{
    app::{EmailSchema, FormValidation, PhoneSchema, VerificationValidation, VerifyPassword},
    entities::{conversation, user_conversation},
};

#[derive(Debug)]
enum UserValidation {
    NoUser,
    SerializationError,
}

cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {

    use super::entities::prelude::*;
    use super::entities::*;
    use sea_orm::*;
    use sea_orm::sea_query::OnConflict;

struct RetrieveConversations;

    #[derive(Debug, sea_orm::FromQueryResult, Serialize, Clone)]
    struct MessageInfo {
        conversation_id: i32,
    }

    #[derive(Debug, sea_orm::FromQueryResult)]
    struct ConversationInfo {
        conversation_id: Option<i32>,
        user_ids: Option<i32>,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>
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


impl RetrieveConversations {

    async fn retrieve_user_conversations(user: &UserLogin, data: &sea_orm::DatabaseConnection) -> Vec<MessageInfo> {
                UserConversation::find()
                    .filter(user_conversation::server::Column::UserIds.eq(user.id))
                    .into_model::<MessageInfo>()
                    .all(data)
                    .await
                    .unwrap()
    }

    async fn retrieve_associated_users(user: UserLogin, data: &sea_orm::DatabaseConnection, condition: sea_orm::Condition) -> Vec<FacingMessageInfo> {

                let associated_users = UserConversation::find()
                    .filter(condition).inner_join(Users)
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
                    .into_model::<ConversationInfo>()
                    .all(data)
                    .await
                    .unwrap();

                println!("Associated Users {:?}", associated_users);
                associated_users
                    .iter()
                    .filter_map(|conversation_info| {
                        if conversation_info.user_ids.unwrap() != user.id && associated_users.iter().any(|conversation|
                            conversation.conversation_id == conversation_info.conversation_id && user.id == conversation.user_ids.unwrap())
                        {
                            Some(FacingMessageInfo {
                                conversation_id: conversation_info.conversation_id,
                                user_ids: conversation_info.user_ids,
                                first_name: conversation_info.first_name.clone(),
                                last_name: conversation_info.last_name.clone(),
                                email: conversation_info.email.clone()
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            }

            async fn retrieve_messages(conversations: &Vec<MessageInfo>, data: &sea_orm::DatabaseConnection) -> Vec<MessageStructFacing> {
                let mut condition: Condition = Condition::any();
                for conversation in conversations {
                    condition = condition.add(message::server::Column::MessageConversationId.eq(conversation.conversation_id));
                }

                Message::find().filter(condition).inner_join(Users).columns::<users::server::Column, Vec<_>>(vec![
                        crate::entities::users::server::Column::FirstName,
                        crate::entities::users::server::Column::LastName,
                ])
                    .order_by_asc(message::server::Column::MessageCreatedAt).into_model::<MessageStruct>().all(data)
                    .await.unwrap().into_iter().map(Into::into).collect()
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
                    .all(data).await.unwrap().into_iter().map(Into::into).collect()

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

}
}
}

#[cfg(feature = "ssr")]
fn evaluate_user(user: Option<actix_identity::Identity>) -> Result<UserLogin, UserValidation> {
    let returned_user: UserLogin;
    match &user.unwrap().id() {
        Ok(val) => match serde_json::from_str(val) {
            Ok(val) => {
                returned_user = val;
                Ok(returned_user)
            }
            Err(e) => {
                println!("Serialization error occured with: {e}");
                Err(UserValidation::SerializationError)
            }
        },
        Err(_) => Err(UserValidation::NoUser),
    }
}

#[server(SignUp, "/api", "Url")]
pub async fn sign_up(
    cx: Scope,
    form: crate::app::SignupSchema,
) -> Result<crate::app::FormValidation, ServerFnError> {
    use super::entities::{prelude::*, *};
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    use rand::Rng;
    use sea_orm::*;

    let mut struct_vector: Vec<Box<dyn validator::Validate>> = Vec::new();

    struct_vector.push(Box::new(form.first_name.clone()));
    struct_vector.push(Box::new(form.last_name.clone()));
    struct_vector.push(Box::new(form.email.clone()));
    struct_vector.push(Box::new(form.password.clone()));
    struct_vector.push(Box::new(form.phone_number.clone()));

    if struct_vector.iter().any(|item| item.validate().is_err()) {
        Ok(crate::app::FormValidation::Error)
    } else {
        // if there is an email entry, then return

        leptos_actix::extract(
            cx,
            move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>| {
                let form = form.clone();
                let entry = form.email.entry.clone();

                async move {
                    if Users::find()
                        .filter(users::server::Column::Email.eq(entry.clone()))
                        .one(&data.lock().await.connection)
                        .await
                        .unwrap()
                        .is_some()
                    {
                        Ok(crate::app::FormValidation::EmailPresent)
                    } else if Users::find()
                        .filter(
                            users::server::Column::PhoneNumber.eq(form
                                .phone_number
                                .entry
                                .replace('+', "")
                                .parse::<i64>()
                                .unwrap()),
                        )
                        .one(&data.lock().await.connection)
                        .await
                        .unwrap()
                        .is_some()
                    {
                        Ok(super::app::FormValidation::PhonePresent)
                    } else {
                        let special_characters = "!@#$%^&*";

                        // Generate a random 15-letter string with lowercase, uppercase, and special characters
                        let mut rng = rand::thread_rng();
                        let random_string: String = (0..15)
                            .map(|_| {
                                let charset: Vec<u8> = match rng.gen_range(0..3) {
                                    0 => (b'a'..=b'z').collect(),
                                    1 => (b'A'..=b'Z').collect(),
                                    _ => special_characters.bytes().collect(),
                                };
                                char::from(charset[rng.gen_range(0..charset.len())])
                            })
                            .collect();

                        let new_user = temp_users::server::ActiveModel {
                            first_name: ActiveValue::Set(form.first_name.entry.clone()),
                            last_name: ActiveValue::Set(form.last_name.entry),
                            email: ActiveValue::Set(form.email.entry.clone()),
                            phone_number: ActiveValue::Set(
                                form.phone_number
                                    .entry
                                    .replace('+', "")
                                    .parse::<i64>()
                                    .unwrap(),
                            ),
                            password: ActiveValue::Set({
                                let salt = SaltString::generate(&mut OsRng);
                                let argon2 = Argon2::default();
                                argon2
                                    .hash_password(form.password.entry.as_bytes(), &salt)
                                    .unwrap()
                                    .to_string()
                            }),
                            verification: {
                                // Define the special characters to include in the random string

                                ActiveValue::Set(random_string.clone())
                            },
                            time: ActiveValue::Set(chrono::Utc::now()),
                            ..Default::default()
                        };
                        if TempUsers::insert(new_user)
                            .exec(&data.lock().await.connection)
                            .await
                            .is_ok()
                        {
                            Ok(super::app::FormValidation::Success {
                                random_string: Some(random_string),
                            })
                        } else {
                            Ok(super::app::FormValidation::Error)
                        }
                    }
                }
            },
        )
        .await
        .unwrap()
    }
}

#[server(Validate, "/api", "Url")]
pub async fn cred_validation(
    cx: Scope,
    email: Option<EmailSchema>,
    phone_number: Option<PhoneSchema>,
) -> Result<crate::app::FormValidation, ServerFnError> {
    use super::entities::{prelude::*, *};
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>| {
            let email = email.clone();
            let phone_number = phone_number.clone();
            async move {
                let db = &data.lock().await.connection;
                if let Some(email) = email {
                    if TempUsers::find()
                        .filter(temp_users::server::Column::Email.eq(email.entry.clone()))
                        .one(db)
                        .await
                        .unwrap()
                        .is_some()
                        || Users::find()
                            .filter(users::server::Column::Email.eq(email.entry))
                            .one(db)
                            .await
                            .unwrap()
                            .is_some()
                    {
                        Ok(FormValidation::EmailPresent)
                    } else {
                        Ok(FormValidation::Success {
                            random_string: None,
                        })
                    }
                } else if TempUsers::find()
                    .filter(
                        temp_users::server::Column::PhoneNumber.eq(phone_number
                            .clone()
                            .unwrap()
                            .entry
                            .replace('+', "")
                            .parse::<i64>()
                            .unwrap()),
                    )
                    .one(db)
                    .await
                    .unwrap()
                    .is_some()
                    || Users::find()
                        .filter(
                            users::server::Column::PhoneNumber.eq(phone_number
                                .clone()
                                .unwrap()
                                .entry
                                .replace('+', "")
                                .parse::<i64>()
                                .unwrap()),
                        )
                        .one(db)
                        .await
                        .unwrap()
                        .is_some()
                {
                    Ok(FormValidation::PhonePresent)
                } else {
                    Ok(FormValidation::Success {
                        random_string: None,
                    })
                }
            }
        },
    )
    .await
    .unwrap()
}

#[server(VerifyEmail, "/api", "Url")]
pub async fn send_verification_email(
    email: String,
    first_name: String,
    random_string: String,
) -> Result<String, ServerFnError> {
    match crate::emailing::email_client::send_email(email, first_name, random_string) {
        Ok(_) => Ok(String::from("Successful Signup")),
        Err(e) => Ok(format!("Error at sending verification email: {e}")),
    }
}

#[server(ConfirmSubscription, "/api", "Url")]
pub async fn confirm_subscription(
    cx: Scope,
    email: String,
    input: String,
) -> Result<VerificationValidation, ServerFnError> {
    use super::entities::{prelude::*, *};
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>| {
            let email = email.clone();
            let input = input.clone();
            println!("{input}");
            async move {
                let db = &data.lock().await.connection;
                if let Ok(user) = TempUsers::find()
                    .filter(temp_users::server::Column::Email.eq(email))
                    .one(&db.clone())
                    .await
                    .map_err(|_| VerificationValidation::EmailNotPresent)
                {
                    let user = user.unwrap();
                    if user.verification.trim().replace('"', "") == input.trim().replace('"', "") {
                        let registered_user = users::server::ActiveModel {
                            first_name: ActiveValue::Set(user.first_name.clone()),
                            last_name: ActiveValue::Set(user.last_name.clone()),
                            email: ActiveValue::Set(user.email.clone()),
                            phone_number: ActiveValue::Set(user.phone_number),
                            password: ActiveValue::Set(user.password.clone()),
                            ..Default::default()
                        };

                        println!("Inserting into db");
                        if Users::insert(registered_user.clone())
                            .exec(&db.clone())
                            .await
                            .is_ok()
                        {
                            println!("User insert route taken");
                            user.delete(&db.clone()).await.unwrap();
                            Ok(VerificationValidation::Success)
                        } else {
                            println!("{}", "error route taken");
                            Ok(super::app::VerificationValidation::ServerError)
                        }
                    } else {
                        Ok(VerificationValidation::IncorrectValidationCode)
                    }
                } else {
                    Ok(VerificationValidation::ServerError)
                }
            }
        },
    )
    .await
    .unwrap()
}

#[server(Login, "/api", "Url")]
pub async fn login(
    cx: Scope,
    email: String,
    password: String,
) -> Result<VerifyPassword, ServerFnError> {
    use super::entities::{prelude::*, *};
    use actix_identity::Identity;
    use actix_web::HttpMessage;
    use actix_web::HttpRequest;
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    use sea_orm::*;

    log!("retrieving request");
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              request: HttpRequest| {
            log!("retrieved request");
            let email = email.clone();
            let password = password.clone();
            async move {
                let db = &data.lock().await.connection;
                if let Some(user) = Users::find()
                    .filter(users::server::Column::Email.eq(email.clone()))
                    .one(db)
                    .await
                    .unwrap()
                {
                    let parsed_hash = PasswordHash::new(&user.password).unwrap();
                    match Argon2::default()
                        .verify_password(password.as_bytes(), &parsed_hash)
                        .is_ok()
                    {
                        true => {
                            Identity::login(
                                &request.extensions(),
                                serde_json::to_string_pretty(&UserLogin {
                                    id: user.id,
                                    email: user.email,
                                })
                                .unwrap(),
                            )
                            .unwrap();
                            Ok(VerifyPassword::Success)
                        }
                        false => Ok(VerifyPassword::IncorrectCredentials),
                    }
                } else {
                    Ok(VerifyPassword::IncorrectCredentials)
                }
            }
        },
    )
    .await
    .unwrap()
}

#[server(LoginStatus, "/api", "Url")]
pub async fn login_status(cx: Scope) -> Result<UserLogin, ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(cx, move |user: Option<Identity>| async {
        let user_evaluation = evaluate_user(user);
        let user: UserLogin;
        match user_evaluation {
            Ok(val) => {
                user = val;
            }
            Err(_) => {
                todo!()
            }
        }
        Ok(user)
    })
    .await
    .unwrap()
}

#[server(Redirect, "/api", "Url")]
pub async fn redirect(cx: Scope) -> Result<bool, ServerFnError> {
    use actix_identity::Identity;
    leptos_actix::extract(
        cx,
        move |user: Option<Identity>| async move { user.is_none() },
    )
    .await
}

// #[cfg(feature = "ssr")]
#[server(GetUsers, "/api", "Url")]
pub async fn get_users(cx: Scope) -> Result<Vec<UserModel>, ServerFnError> {
    use super::entities::prelude::*;
    use super::entities::users;
    use sea_orm::*;

    Ok(leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<actix_identity::Identity>| {
            let user = evaluate_user(user);
            async move {
                let data = &data.lock().await.connection;
                Users::find()
                    .order_by_desc(users::server::Column::Id)
                    .filter(users::server::Column::Id.ne(user.unwrap().id))
                    .all(data)
                    .await
                    .unwrap()
            }
        },
    )
    .await
    .unwrap()
    .iter()
    .map(|val| {
        let val = val.clone();
        UserModel {
            email: val.email,
            first_name: val.first_name,
            last_name: val.last_name,
            id: val.id,
            phone_number: val.phone_number,
        }
    })
    .rev()
    .collect())
}

#[server(GetConversations, "/api", "Url")]
pub async fn get_conversations(cx: Scope) -> Result<Vec<MergedConversation>, ServerFnError> {
    use actix_identity::Identity;
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let user: UserLogin = serde_json::from_str(&user.unwrap().id().unwrap()).unwrap();

            async move {
                let data = &data.lock().await.connection;
                let conversations =
                    RetrieveConversations::retrieve_user_conversations(&user, data).await;

                println!("Conversation {:?}", conversations);
                let mut condition = Condition::any();
                for conversation in &conversations {
                    condition = condition.add(
                        user_conversation::server::Column::ConversationId
                            .eq(conversation.conversation_id),
                    );
                }

                let users =
                    RetrieveConversations::retrieve_associated_users(user, data, condition).await;

                println!("Users {:?}", users);

                let messages = RetrieveConversations::retrieve_messages(&conversations, data).await;

                let seen_messages = RetrieveConversations::retrieve_seen(&messages, data).await;

                let mut vec_merged_conversation = Vec::new();

                for conversation in &conversations {
                    println!("{}", conversation.conversation_id);
                    println!("{:?}", users);

                    let users = users
                        .iter()
                        .find(|user| user.conversation_id.unwrap() == conversation.conversation_id)
                        .unwrap();

                    let merged_messages: Vec<MergedMessages> = messages
                        .iter()
                        .map(|messages| MergedMessages {
                            message_conversation_id: messages.message_conversation_id,
                            message_id: messages.message_id,
                            message_body: messages.message_body.clone(),
                            message_image: messages.message_image.clone(),
                            message_sender_id: messages.message_sender_id,
                            seen_status: seen_messages
                                .clone()
                                .into_iter()
                                .filter(|seen_messages| {
                                    seen_messages.message_id.unwrap() == messages.message_id
                                })
                                .collect(),
                            created_at: messages.message_created_at.to_string(),
                            first_name: messages.first_name.clone(),
                            last_name: messages.last_name.clone(),
                        })
                        .collect::<Vec<MergedMessages>>();

                    vec_merged_conversation.push(MergedConversation {
                        conversation_id: conversation.conversation_id,
                        conversation: ConversationInner {
                            user_ids: users.user_ids.unwrap(),
                            last_name: users.last_name.clone().unwrap(),
                            first_name: users.first_name.clone().unwrap(),
                            messages: merged_messages
                                .into_iter()
                                .filter(|message| {
                                    message.message_conversation_id == conversation.conversation_id
                                })
                                .collect(),
                        },
                    });
                    // vec_merged_conversation.push(value)
                }
                Ok(vec_merged_conversation)
            }
        },
    )
    .await
    .unwrap()
    // Ok(true)
}

#[server(Logout, "/api", "Url")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(cx, move |user: Option<Identity>| async {
        user.unwrap().logout()
    })
    .await
}

#[server(ConversationAction, "/api", "Url")]
pub async fn conversation_action(cx: Scope, other_user: i32) -> Result<(), ServerFnError> {
    use crate::entities::prelude::*;
    use actix_identity::Identity;
    use iter_tools::prelude::Itertools;
    use sea_orm::prelude::*;
    use sea_orm::*;

    #[derive(FromQueryResult, PartialEq, Eq, Hash)]
    struct ExtractedConversation {
        conversation_id: i32,
    }

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let user_evaluation = evaluate_user(user);
            let user: UserLogin;
            match user_evaluation {
                Ok(val) => {
                    user = val;
                }
                Err(_) => {
                    todo!()
                }
            }

            async move {
                let data = &data.lock().await.connection;
                let existing_conversation = UserConversation::find()
                    .select_only()
                    .column(user_conversation::server::Column::ConversationId)
                    .filter(
                        Condition::any()
                            .add(user_conversation::server::Column::UserIds.eq(user.id))
                            .add(user_conversation::server::Column::UserIds.eq(other_user)),
                    )
                    .into_model::<ExtractedConversation>()
                    .all(data)
                    .await
                    .unwrap();

                if existing_conversation.iter().all_unique() || existing_conversation.len().eq(&0) {
                    log!("Inserting Conversation");
                    let conversation = Conversation::insert(conversation::server::ActiveModel {
                        name: ActiveValue::Set(String::from("New Conversation")),
                        is_group: ActiveValue::Set(0),
                        ..Default::default()
                    })
                    .exec(data)
                    .await
                    .unwrap();

                    for user in [user.id, other_user].iter() {
                        UserConversation::insert(user_conversation::server::ActiveModel {
                            user_ids: ActiveValue::Set(*user),
                            conversation_id: ActiveValue::Set(conversation.last_insert_id),
                        })
                        .exec(data)
                        .await
                        .unwrap();
                    }
                } else {
                    log!("Existing Conversation Found");
                    let conversation = existing_conversation.iter().duplicates().next().unwrap();
                }
            }
        },
    )
    .await
    .unwrap();
    Ok(())
}

#[server(ValidateConversation, "/api", "Url")]
pub async fn validate_conversation(
    cx: Scope,
    desired_conversation_id: i32,
) -> Result<Vec<ConversationMeta>, ServerFnError> {
    use crate::entities::prelude::*;
    use actix_identity::Identity;
    use sea_orm::prelude::*;
    use sea_orm::Condition;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user_evaluation = evaluate_user(user);
                let user: UserLogin;
                match user_evaluation {
                    Ok(val) => {
                        user = val;
                    }
                    Err(_) => {
                        todo!()
                    }
                }

                let user_conversations =
                    RetrieveConversations::retrieve_user_conversations(&user, data).await;

                let mut condition: Condition = Condition::any();

                for conversation in &user_conversations {
                    condition = condition
                        .add(conversation::server::Column::Id.eq(conversation.conversation_id))
                }

                let conversations = Conversation::find()
                    .filter(condition)
                    .all(data)
                    .await
                    .unwrap();

                let other_condition = Condition::all().add(
                    user_conversation::server::Column::ConversationId.eq(desired_conversation_id),
                );

                let other_users =
                    RetrieveConversations::retrieve_associated_users(user, data, other_condition)
                        .await;

                conversations
                    .into_iter()
                    .filter_map(|conversation| {
                        if conversation.id == desired_conversation_id {
                            Some(ConversationMeta {
                                id: conversation.id,
                                last_message_at: conversation.last_message_at.to_string(),
                                created_at: conversation.created_at.to_string(),
                                name: conversation.name,
                                is_group: conversation.is_group,
                                count: user_conversations.len(),
                                other_users: other_users
                                    .iter()
                                    .map(|users| {
                                        (
                                            users.first_name.clone().unwrap()
                                                + " "
                                                + &users.last_name.clone().unwrap(),
                                            users.email.clone().unwrap(),
                                        )
                                    })
                                    .collect(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        },
    )
    .await
}

#[server(ViewMessages, "/api", "Url")]
pub async fn view_messages(
    cx: Scope,
    desired_conversation_id: i32,
) -> Result<Vec<MergedMessages>, ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user_evaluation = evaluate_user(user);
                let user: UserLogin;
                match user_evaluation {
                    Ok(val) => {
                        user = val;
                    }
                    Err(_) => {
                        todo!()
                    }
                }

                let messages = RetrieveConversations::retrieve_messages(
                    &vec![MessageInfo {
                        conversation_id: desired_conversation_id,
                    }],
                    data,
                )
                .await;

                let seen_messages = RetrieveConversations::retrieve_seen(&messages, data).await;

                messages
                    .iter()
                    .map(|message| MergedMessages {
                        message_conversation_id: message.message_conversation_id,
                        message_id: message.message_id,
                        message_body: message.message_body.clone(),
                        created_at: message.message_created_at.to_string(),
                        message_sender_id: message.message_sender_id,
                        message_image: message.message_image.clone(),
                        seen_status: seen_messages
                            .clone()
                            .into_iter()
                            .filter(|seen_messages| {
                                seen_messages.message_id.unwrap() == message.message_id
                            })
                            .collect(),
                        first_name: message.first_name.clone(),
                        last_name: message.last_name.clone(),
                    })
                    .collect::<Vec<_>>()
            }
        },
    )
    .await
}

#[server(AssociatedConversation, "/api", "Url")]
pub async fn associated_conversation(
    cx: Scope,
    other_user: i32,
) -> Result<Option<i32>, ServerFnError> {
    use actix_identity::Identity;
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user_evaluation = evaluate_user(user);
                let user: UserLogin;
                match user_evaluation {
                    Ok(val) => {
                        user = val;
                    }
                    Err(_) => {
                        todo!()
                    }
                }

                let condition = Condition::all().add(
                    user_conversation::server::Column::UserIds
                        .eq(other_user)
                        .add(user_conversation::server::Column::UserIds.eq(user.id)),
                );

                RetrieveConversations::retrieve_associated_users(user, data, condition)
                    .await
                    .first()
                    .unwrap()
                    .conversation_id
            }
        },
    )
    .await
}

#[server(HandleMessageInput, "/api", "Url")]
pub async fn handle_message_input(
    cx: Scope,
    conversation_id: i32,
    body: Option<String>,
    image: Option<Vec<u8>>,
) -> Result<(), ServerFnError> {
    use crate::entities::message;
    use actix_identity::Identity;
    use sea_orm::prelude::*;

    if body.is_none() && image.is_none() {
        return Err(server_fn::ServerFnError::MissingArg(String::from(
            "Body Missing",
        )));
    }
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let body = body.clone();
            let image = image.clone();
            async move {
                let data = &data.lock().await.connection;
                let user_evaluation = evaluate_user(user);
                let user: UserLogin;
                match user_evaluation {
                    Ok(val) => {
                        user = val;
                    }
                    Err(_) => {
                        todo!()
                    }
                }
                let mut image_location: Option<String> = Default::default();
                image.map(|image_vec| {
                    println!("image is some");
                    let current_time = std::time::UNIX_EPOCH
                        .elapsed()
                        .unwrap()
                        .as_secs()
                        .to_string();

                    if std::fs::metadata("./upload").is_err() {
                        std::fs::create_dir_all("./upload").unwrap();
                    };

                    std::fs::write("./upload/".to_string() + &current_time + ".png", image_vec)
                        .ok();
                    image_location = Some("/upload/".to_string() + &current_time + ".png")
                });

                AppendDatabase::insert_messages(
                    data,
                    message::server::ActiveModel {
                        message_body: sea_orm::ActiveValue::Set(body),
                        message_sender_id: sea_orm::ActiveValue::Set(user.id),
                        message_image: sea_orm::ActiveValue::Set(image_location),
                        message_conversation_id: sea_orm::ActiveValue::Set(conversation_id),
                        ..Default::default()
                    },
                )
                .await
            }
        },
    )
    .await
}

#[server(FindImage, "/api", "Url")]
pub async fn find_image(cx: Scope, image_path: String) -> Result<ImageAvailability, ServerFnError> {
    Ok(
        match std::fs::metadata(
            std::env::current_dir()
                .unwrap()
                .join(format!("upload/{}", image_path.split('/').last().unwrap())),
        ) {
            Ok(_) => ImageAvailability::Found,
            Err(_) => ImageAvailability::Missing,
        },
    )
}

#[server(HandleSeen, "/api", "Url")]
pub async fn handle_seen(cx: Scope, conversation_id: i32) -> Result<(), ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user_evaluation = evaluate_user(user);
                let user: UserLogin;
                match user_evaluation {
                    Ok(val) => {
                        user = val;
                    }
                    Err(_) => {
                        todo!()
                    }
                }
                let messages: Vec<i32> = RetrieveConversations::retrieve_messages(
                    &vec![MessageInfo { conversation_id }],
                    data,
                )
                .await
                .iter()
                .map(|messages| messages.message_id)
                .collect();

                AppendDatabase::insert_seen(data, messages, user.id).await
            }
        },
    )
    .await
}
