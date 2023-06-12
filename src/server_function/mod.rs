use std::vec;

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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConversationMeta {
        pub id: i32,
        pub last_message_at: String,
        pub created_at: String,
        pub name: String,
        pub is_group: i8,
        pub count: usize
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergedConversation {
    pub conversation_id: i32,
    pub conversation: ConversationInner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationInner {
    pub user_ids: i32,
    pub first_name: String,
    pub last_name: String,
    pub messages: Vec<MergedMessages>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergedMessages {
    pub message_id: i32,
    pub message_body: Option<String>,
    pub message_image: Option<String>,
    pub message_sender_id: i32,
    pub seen_status: bool,
    pub created_at: String,
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
    }

impl RetrieveConversations {

    async fn retrieve_user_conversations(user: UserLogin, data: &sea_orm::DatabaseConnection) -> Vec<MessageInfo> {
    use super::entities::prelude::*;
    use sea_orm::*;
                UserConversation::find()
                    .filter(user_conversation::server::Column::UserIds.eq(user.id))
                    .into_model::<MessageInfo>()
                    .all(data)
                    .await
                    .unwrap()
    }

    async fn retrieve_associated_users(user: UserLogin, data: &sea_orm::DatabaseConnection, condition: sea_orm::Condition) -> Vec<FacingMessageInfo> {
    use super::entities::prelude::*;
    use sea_orm::*;

                let associated_users = UserConversation::find()
                    .filter(condition).inner_join(Users)
                    // .join(
                    //     JoinType::LeftJoin,
                    //     conversation::server::Entity::belongs_to(user_conversation::server::Entity)
                    //         .to(user_conversation::server::Column::ConversationId)
                    //         .from(conversation::server::Column::Id)
                    //         .into(),
                    // )
                    .columns::<user_conversation::server::Column, Vec<_>>(vec![
                        user_conversation::server::Column::UserIds,
                        user_conversation::server::Column::ConversationId,
                    ])
                    .columns::<crate::entities::users::server::Column, Vec<_>>(vec![
                        crate::entities::users::server::Column::Id,
                        crate::entities::users::server::Column::FirstName,
                        crate::entities::users::server::Column::LastName,
                    ])
                    // .join(
                    //     JoinType::LeftJoin,
                    //     crate::entities::user_conversation::server::Relation::Users.def(),
                    // )
                    .into_model::<ConversationInfo>()
                    .all(data)
                    .await
                    .unwrap();
                println!("Associated Users {:?}", associated_users);
                associated_users
                    .iter()
                    .filter_map(|conversation_info| {
                        if conversation_info.user_ids.unwrap() != user.id {
                            Some(FacingMessageInfo {
                                conversation_id: conversation_info.conversation_id,
                                user_ids: conversation_info.user_ids,
                                first_name: conversation_info.first_name.clone(),
                                last_name: conversation_info.last_name.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            }

            async fn retrieve_messages(conversations: &Vec<MessageInfo>, data: &sea_orm::DatabaseConnection) -> Vec<crate::entities::message::server::Model> {
                use sea_orm::*;
                use crate::entities::prelude::*;
                use crate::entities::message;

                let mut condition: Condition = Condition::any();
                for conversation in conversations {
                    condition = condition.add(message::server::Column::MessageConversationId.eq(conversation.conversation_id));
                }

                Message::find().filter(condition)
                    .order_by_asc(message::server::Column::MessageCreatedAt).all(data).await.unwrap()
            }

            async fn retrieve_seen(messages: &Vec<crate::entities::message::server::Model>, data: &sea_orm::DatabaseConnection) -> Vec<crate::entities::seen_messages::server::Model> {
                use sea_orm::*;
                use crate::entities::prelude::*;
                use crate::entities::seen_messages;

                let mut condition: Condition = Condition::any();
                for message in messages {
                    condition = condition.add(seen_messages::server::Column::MessageId.eq(message.message_id));
                }

                SeenMessages::find().filter(condition)
                    .all(data).await.unwrap()

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
pub async fn login_status(cx: Scope) -> Result<String, ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(cx, move |user: Option<Identity>| async {
        Ok(user.unwrap().id().unwrap())
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
    use super::entities::conversation;

    use actix_identity::Identity;
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let user: UserLogin = serde_json::from_str(&user.unwrap().id().unwrap()).unwrap();

            async move {
                let data = &data.lock().await.connection;
                let conversation =
                    RetrieveConversations::retrieve_user_conversations(user.clone(), data).await;

                println!("Conversation {:?}", conversation);
                let mut condition = Condition::any();
                for conversation_id in &conversation {
                    condition = condition.add(
                        user_conversation::server::Column::ConversationId
                            .eq(conversation_id.conversation_id),
                    );
                }

                let users =
                    RetrieveConversations::retrieve_associated_users(user, data, condition).await;

                println!("Users {:?}", users);

                let messages = RetrieveConversations::retrieve_messages(&conversation, data).await;

                let seen_messages = RetrieveConversations::retrieve_seen(&messages, data).await;

                let mut vec_merged_conversation = Vec::new();

                for conversation in &conversation {
                    println!("{}", conversation.conversation_id);
                    println!("{:?}", users);
                    let users = users
                        .iter()
                        .find(|user| user.conversation_id.unwrap() == conversation.conversation_id)
                        .unwrap();
                    let merged_messages: Vec<MergedMessages> = messages
                        .iter()
                        .map(|messages| MergedMessages {
                            message_id: messages.message_id,
                            message_body: Some(messages.message_body.clone()),
                            message_image: messages.message_image.clone(),
                            message_sender_id: messages.message_sender_id,
                            seen_status: seen_messages
                                .iter()
                                .any(|message| message.message_id == messages.message_id),
                            created_at: messages.message_created_at.to_string(),
                        })
                        .collect::<Vec<MergedMessages>>();

                    vec_merged_conversation.push(MergedConversation {
                        conversation_id: conversation.conversation_id,
                        conversation: ConversationInner {
                            user_ids: users.user_ids.unwrap(),
                            last_name: users.last_name.clone().unwrap(),
                            first_name: users.first_name.clone().unwrap(),
                            messages: merged_messages,
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
    use actix_identity::Identity;
    use crate::entities::prelude::*;
    use sea_orm::Condition;
    use sea_orm::prelude::*;

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
                    RetrieveConversations::retrieve_user_conversations(user, data).await;

                let mut condition: Condition = Condition::any();

                for conversation in &user_conversations {
                        condition = condition.add(conversation::server::Column::Id.eq(conversation.conversation_id))
                }

                let conversations = Conversation::find().filter(condition).all(data).await.unwrap();

                conversations
                    .into_iter()
                    .filter_map(|conversation| {if conversation.id == desired_conversation_id {
                            Some(
                            ConversationMeta {
                                id: conversation.id,
                                last_message_at: conversation.last_message_at.to_string(),
                                created_at: conversation.created_at.to_string(),
                                name: conversation.name,
                                is_group: conversation.is_group,
                                count: user_conversations.iter().count()
                            }
                        )} else {
                            None
                        }
                    }).collect()
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
                        message_id: message.message_id,
                        message_body: Some(message.message_body.clone()),
                        created_at: message.message_created_at.to_string(),
                        message_sender_id: message.message_sender_id,
                        message_image: message.message_image.clone(),
                        seen_status: seen_messages
                            .iter()
                            .any(|seen| seen.message_id == message.message_id),
                    })
                    .collect::<Vec<_>>()
            }
        },
    )
    .await
}
