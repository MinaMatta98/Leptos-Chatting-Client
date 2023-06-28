use iter_tools::Itertools;
use leptos::*;
use serde::{Deserialize, Serialize};

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

use crate::app::{EmailSchema, PhoneSchema, VerificationValidation, VerifyPassword};

#[cfg(feature = "ssr")]
use crate::entities::{conversation, user_conversation};
#[cfg(feature = "ssr")]
use crate::app::FormValidation;

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

    let struct_vector: Vec<Box<dyn validator::Validate>> = vec![
        Box::new(form.first_name.clone()),
        Box::new(form.last_name.clone()),
        Box::new(form.email.clone()),
        Box::new(form.password.clone()),
        Box::new(form.phone_number.clone()),
    ];

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
                                    .chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .collect::<String>()
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
                            Ok(VerificationValidation::Success)
                        } else {
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
                                    email: user.email.clone(),
                                    first_name: user.first_name.clone(),
                                    last_name: user.last_name.clone(),
                                })
                                .unwrap(),
                            )
                            .unwrap();
                            Ok(VerifyPassword::Success(UserLogin {
                                    id: user.id,
                                    email: user.email,
                                    first_name: user.first_name,
                                    last_name: user.last_name,
                                }))
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
        let user = match UserLogin::evaluate_user(user) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
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
            async move {
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let data = &data.lock().await.connection;
                Ok(Users::find()
                    .order_by_asc(users::server::Column::Id)
                    .filter(users::server::Column::Id.ne(user.id))
                    .all(data)
                    .await
                    .unwrap())
            }
        },
    )
    .await
    .unwrap()
    .unwrap()
    .into_iter()
    .map_into()
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
            async move {
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let data = &data.lock().await.connection;
                let conversations =
                    RetrieveConversations::retrieve_user_conversations(&user, data).await;

                let mut condition = Condition::any();
                for conversation in &conversations {
                    condition = condition.add(
                        user_conversation::server::Column::ConversationId
                            .eq(conversation.conversation_id),
                    );
                }

                let users =
                    RetrieveConversations::retrieve_associated_users(user.clone(), data, condition)
                        .await;

                let messages = RetrieveConversations::retrieve_messages(
                    &conversations
                        .iter()
                        .map(|conversation| conversation.conversation_id)
                        .collect(),
                    data,
                )
                .await;

                let seen_messages = RetrieveConversations::retrieve_seen(&messages, data).await;

                let vec_merged_conversation = conversations
                    .iter()
                    .map(|conversation| {
                        let conversation_id = conversation.conversation_id;
                        let conversation_users = users
                            .iter()
                            .filter(|user| user.conversation_id == conversation_id)
                            .collect_vec();

                        let merged_messages: Vec<MergedMessages> = messages
                            .iter()
                            .filter(|message| message.message_conversation_id == conversation_id)
                            .map(|messages| {
                                let seen_status = seen_messages
                                    .iter()
                                    .filter(|seen_messages| {
                                        seen_messages.message_id.unwrap() == messages.message_id
                                    })
                                    .cloned()
                                    .collect_vec();

                                MergedMessages {
                                    message_conversation_id: messages.message_conversation_id,
                                    message_id: messages.message_id,
                                    message_body: messages.message_body.clone(),
                                    message_image: messages.message_image.clone(),
                                    message_sender_id: messages.message_sender_id,
                                    seen_status,
                                    created_at: messages.message_created_at.to_string(),
                                    first_name: messages.first_name.clone(),
                                    last_name: messages.last_name.clone(),
                                }
                            })
                            .collect();

                        let (last_name, first_name) = conversation_users
                            .iter()
                            .find(|&users| *users.user_ids.first().unwrap() != user.id)
                            .map(|user| (user.last_name.clone(), user.first_name.clone()))
                            .unwrap();

                        let conversation_messages = merged_messages
                            .into_iter()
                            .filter(|message| message.message_conversation_id == conversation_id)
                            .collect();

                        MergedConversation {
                            conversation_id,
                            conversation: ConversationInner {
                                user_ids: conversation_users
                                    .iter()
                                    .rev()
                                    .map(|user| *user.user_ids.first().unwrap())
                                    .collect(),
                                last_name,
                                first_name,
                                name: conversation.name.clone(),
                                is_group: conversation.is_group,
                                messages: conversation_messages,
                            },
                        }
                    })
                    .collect();

                Ok(vec_merged_conversation)
            }
        },
    )
    .await
    .unwrap()
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
pub async fn conversation_action(
    cx: Scope,
    other_users: Vec<i32>,
    is_group: bool,
    name: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::entities::prelude::*;
    use actix_identity::Identity;
    use iter_tools::prelude::Itertools;
    use sea_orm::prelude::*;
    use sea_orm::*;

    #[derive(FromQueryResult, PartialEq, Eq, Hash, Debug)]
    struct ExtractedConversation {
        conversation_id: i32,
    }

    if other_users.len().lt(&2) && is_group {
        return Err(ServerFnError::Args("Not Enough Users Added".to_string()));
    };

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let other_users = other_users.clone();
            let name = name.clone();
            async move {
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let data = &data.lock().await.connection;

                let mut existing_conversation = UserConversation::find()
                    .select_only()
                    .column(user_conversation::server::Column::ConversationId)
                    .column(user_conversation::server::Column::UserIds)
                    .column(conversation::server::Column::Name)
                    .column(conversation::server::Column::Id)
                    .right_join(Conversation);

                match is_group {
                    true => {
                        existing_conversation = existing_conversation
                            .filter(conversation::server::Column::Name.is_not_null())
                    }
                    false => {
                        existing_conversation = existing_conversation
                            .filter(conversation::server::Column::Name.is_null())
                    }
                };

                let resolved_conversations = existing_conversation
                    .filter(
                        Condition::any().add(
                            user_conversation::server::Column::UserIds
                                .is_in(other_users.clone())
                                .add(user_conversation::server::Column::UserIds.eq(user.id)),
                        ),
                    )
                    .into_model::<ExtractedConversation>()
                    .all(data)
                    .await
                    .unwrap();

                if resolved_conversations.iter().all_unique() || resolved_conversations.len().eq(&0)
                {
                    match is_group {
                        false => {
                            log!("Inserting Conversation");
                            let conversation =
                                Conversation::insert(conversation::server::ActiveModel {
                                    is_group: ActiveValue::Set(0),
                                    name: ActiveValue::Set(None),
                                    ..Default::default()
                                })
                                .exec(data)
                                .await
                                .unwrap();

                            for user in [user.id, *other_users.first().unwrap()].iter() {
                                UserConversation::insert(user_conversation::server::ActiveModel {
                                    user_ids: ActiveValue::Set(*user),
                                    conversation_id: ActiveValue::Set(conversation.last_insert_id),
                                })
                                .exec(data)
                                .await
                                .unwrap();
                            }
                        }
                        true => {
                            let conversation =
                                Conversation::insert(conversation::server::ActiveModel {
                                    is_group: ActiveValue::Set(1),
                                    name: ActiveValue::Set(name),
                                    ..Default::default()
                                })
                                .exec(data)
                                .await
                                .unwrap();

                            let mut vec_users = Vec::new();
                            [vec![user.id], other_users]
                                .iter()
                                .flatten()
                                .for_each(|&user| {
                                    vec_users.push(user_conversation::server::ActiveModel {
                                        user_ids: ActiveValue::Set(user),
                                        conversation_id: ActiveValue::Set(
                                            conversation.last_insert_id,
                                        ),
                                    })
                                });

                            UserConversation::insert_many(vec_users)
                                .exec(data)
                                .await
                                .unwrap();
                        }
                    }
                    Ok(())
                } else {
                    log!("Existing Conversation Found");
                    Ok(())
                }
            }
        },
    )
    .await
    .unwrap()
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
    use iter_tools::Itertools;
    use sea_orm::prelude::*;
    use sea_orm::Condition;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let user_conversations =
                    RetrieveConversations::retrieve_user_conversations(&user, data).await;

                if !user_conversations
                    .iter()
                    .any(|conversation| conversation.conversation_id == desired_conversation_id)
                {
                    return Err(ServerFnError::ServerError("Access Denied".to_string()));
                };

                let conversations = Conversation::find()
                    .filter(conversation::server::Column::Id.eq(desired_conversation_id))
                    .all(data)
                    .await
                    .unwrap();

                let other_users = RetrieveConversations::retrieve_associated_users(
                    user,
                    data,
                    Condition::any().add(
                        user_conversation::server::Column::ConversationId
                            .eq(desired_conversation_id),
                    ),
                )
                .await;

                Ok(conversations
                    .into_iter()
                    .map(|conversation| ConversationMeta {
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
                                    format!(
                                        "{} {}",
                                        users.first_name.clone(),
                                        &users.last_name.clone()
                                    ),
                                    users.email.clone(),
                                    *users.user_ids.first().unwrap(),
                                )
                            })
                            .sorted()
                            .unique()
                            .collect(),
                    })
                    .collect())
            }
        },
    )
    .await
    .unwrap()
}

#[server(ViewMessages, "/api", "Url")]
pub async fn view_messages(
    cx: Scope,
    desired_conversation_id: i32,
) -> Result<Vec<MergedMessages>, ServerFnError> {
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>| {
            async move {
                let data = &data.lock().await.connection;
                let messages = RetrieveConversations::retrieve_messages(
                        &vec![desired_conversation_id],
                        data
                )
                .await;

                let seen_messages = RetrieveConversations::retrieve_seen(&messages, data).await;

                Ok(messages
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
                    .collect::<Vec<_>>())
            }
        },
    )
    .await
    .unwrap()
}

#[server(AssociatedConversation, "/api", "Url")]
pub async fn associated_conversation(cx: Scope, other_user: i32) -> Result<i32, ServerFnError> {
    use actix_identity::Identity;
    use sea_orm::*;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;

                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let condition = Condition::all()
                    .add(
                        Condition::any()
                            .add(user_conversation::server::Column::UserIds.eq(other_user))
                            .add(user_conversation::server::Column::UserIds.eq(user.id)),
                    )
                    .add(conversation::server::Column::IsGroup.eq(0));

                let conversations =
                    RetrieveConversations::retrieve_associated_users(user.clone(), data, condition)
                        .await;

                let user_conversation = conversations
                    .iter()
                    .filter(|conversations| {
                        *conversations.user_ids.first().unwrap() == user.clone().id
                    })
                    .collect::<Vec<_>>();

                Ok(conversations
                    .iter()
                    .find_map(|conversations| {
                        if *conversations.user_ids.first().unwrap() != user.id
                            && user_conversation.iter().any(|user_conversation| {
                                user_conversation.conversation_id == conversations.conversation_id
                            })
                        {
                            Some(conversations.conversation_id)
                        } else {
                            None
                        }
                    })
                    .unwrap())
            }
        },
    )
    .await
    .unwrap()
}

#[server(HandleMessageInput, "/api", "Url")]
pub async fn handle_message_input(
    cx: Scope,
    conversation_id: i32,
    body: Option<String>,
    image: Option<Vec<u8>>,
) -> Result<Option<String>, ServerFnError> {
    use crate::entities::message;
    use actix_identity::Identity;
    use image::io::Reader as ImageReader;

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
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let mut image_location: Option<String> = Default::default();

                if let Some(image_vec) = image {
                    let current_time = std::time::UNIX_EPOCH
                        .elapsed()
                        .unwrap()
                        .as_secs()
                        .to_string();

                    if std::fs::metadata("./upload").is_err() {
                        std::fs::create_dir_all("./upload").unwrap();
                    };

                    let kind = infer::get(&image_vec).expect("file type is known");
                    let image = if !kind.mime_type().eq("image/png") {
                        let image = ImageReader::new(std::io::Cursor::new(image_vec))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap();

                        turbojpeg::compress_image(
                            &image.into_rgba8(),
                            50,
                            turbojpeg::Subsamp::Sub2x2,
                        )
                        .unwrap()
                        .to_vec()
                    } else {
                        image_vec
                    };
                    std::fs::write("./upload/".to_string() + &current_time + ".png", image).ok();
                    image_location = Some("/upload/".to_string() + &current_time + ".png")
                };

                AppendDatabase::insert_messages(
                    data,
                    message::server::ActiveModel {
                        message_body: sea_orm::ActiveValue::Set(body),
                        message_sender_id: sea_orm::ActiveValue::Set(user.id),
                        message_image: sea_orm::ActiveValue::Set(image_location.clone()),
                        message_conversation_id: sea_orm::ActiveValue::Set(conversation_id),
                        ..Default::default()
                    },
                )
                .await;

                Ok(image_location)
            }
        },
    )
    .await
    .unwrap()
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
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let messages: Vec<i32> =
                    RetrieveConversations::retrieve_messages(&vec![conversation_id], data)
                        .await
                        .iter()
                        .map(|messages| messages.message_id)
                        .collect();
                log!("MESSAGES {messages:?}");
                AppendDatabase::insert_seen(data, messages, user.id).await;
                Ok(())
            }
        },
    )
    .await
    .unwrap()
}

#[server(DeleteConversation, "/api", "Url")]
pub async fn delete_conversations(cx: Scope, conversation_id: i32) -> Result<(), ServerFnError> {
    use actix_identity::Identity;

    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                AppendDatabase::delete_conversation(conversation_id, data, user).await;
                Ok(())
            }
        },
    )
    .await
    .unwrap()
}

#[server(GetUser, "/api", "Url")]
pub async fn get_user(cx: Scope) -> Result<UserModel, ServerFnError> {
    use actix_identity::Identity;
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            async move {
                let data = &data.lock().await.connection;
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };
                Ok(UserLogin::retrieve_user(user, data).await)
            }
        },
    )
    .await
    .unwrap()
}

#[server(UploadImage, "/api", "Url")]
pub async fn upload_user_info(
    cx: Scope,
    image: Option<Vec<u8>>,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<(), ServerFnError> {
    use actix_identity::Identity;
    use image::io::Reader as ImageReader;
    use validator::Validate;
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>,
              user: Option<Identity>| {
            let image = image.clone();
            let first_name = first_name.clone();
            let last_name = last_name.clone();
            async move {
                let data = &data.lock().await.connection;
                let user = match UserLogin::evaluate_user(user) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let mut validation_vec = Vec::new();

                [
                    ("first name".to_string(), first_name.clone()),
                    ("last_name".to_string(), last_name.clone()),
                ]
                .into_iter()
                .for_each(|(entry, name)| {
                    if let Some(name) = name {
                        let schema = crate::app::NameSchema { entry: name };
                        match schema.validate() {
                            Ok(_) => (),
                            Err(e) => validation_vec.push((entry, e)),
                        }
                    }
                });

                if validation_vec.len().gt(&0) {
                    let mut validation_string = String::new();
                    for error in validation_vec {
                        validation_string.push_str(
                            &(format!(
                                "Entry {} failed to register with error {}\n",
                                error.0, error.1
                            )),
                        )
                    }
                    return Err(ServerFnError::Args(format!(
                        "Error occured while validating fields:
                            {validation_string}"
                    )));
                }

                if let Some(image) = image {
                    let kind = infer::get(&image).expect("file type is known");
                    if kind.mime_type() != "image/jpeg" && kind.mime_type() != "image/png" {
                        return Err(ServerFnError::Args(format!("Incorrect Mime Type {}", kind)));
                    };
                    let current_time = std::time::UNIX_EPOCH
                        .elapsed()
                        .unwrap()
                        .as_secs()
                        .to_string();

                    if std::fs::metadata("./images").is_err() {
                        std::fs::create_dir_all("./images").unwrap();
                    };

                    let image_path = "images/".to_string() + &current_time + ".png";
                    let image = if !kind.mime_type().eq("image/png") {
                        let image = ImageReader::new(std::io::Cursor::new(image))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap();

                        turbojpeg::compress_image(
                            &image.into_rgba8(),
                            50,
                            turbojpeg::Subsamp::Sub2x2,
                        )
                        .unwrap()
                        .to_vec()
                    } else {
                        image
                    };
                    std::fs::write(&image_path, image).unwrap();

                    AppendDatabase::modify(
                        user,
                        Some(image_path),
                        data,
                        first_name.clone(),
                        last_name.clone(),
                    )
                    .await;
                } else {
                    AppendDatabase::modify(user, None, data, first_name.clone(), last_name.clone())
                        .await;
                }

                Ok(())
            }
        },
    )
    .await
    .unwrap()
}

#[server(GetIcon, "/api", "Url")]
pub async fn get_icon(cx: Scope, id: i32) -> Result<Option<Vec<u8>>, ServerFnError> {
    use std::io::Read;
    leptos_actix::extract(
        cx,
        move |data: actix_web::web::Data<tokio::sync::Mutex<crate::database::DbConnection>>| {
            async move {
                let data = &data.lock().await.connection;
                let image = RetrieveConversations::retrieve_images(id, data).await;
                if let Some(image) = image {
                         let path = std::env::current_dir()
                             .unwrap_or_default()
                             .join(image);

                         if let Ok(mut file) = std::fs::File::open(path) {
                             let mut buffer = Vec::new();
                             file.read_to_end(&mut buffer).unwrap();
                             Some(buffer)
                        } else {
                             None
                        }
                } else {
                    None
                }
            }
        },
    )
    .await
}

#[server(GetImage, "/api", "Url")]
pub async fn get_image(cx: Scope, path: String) -> Result<Option<Vec<u8>>, ServerFnError> {
    use std::io::Read;
    let mut path = path;
    path.remove(0);
    let path = std::env::current_dir().unwrap().join(path);

    let mut buffer = Vec::new();
    if let Ok(mut file) = std::fs::File::open(path) {
        file.read_to_end(&mut buffer).unwrap();
        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}

#[server(CreateGroupConversation, "/api", "Url")]
pub async fn create_group_conversations(
    cx: Scope,
    other_users: String,
    is_group: bool,
    name: Option<String>,
) -> Result<(), ServerFnError> {
    let other_users_vec: Vec<i32> = other_users
        .split(',')
        .map(|user_ids| user_ids.parse::<i32>().expect("Invalid user selection"))
        .collect();
    conversation_action(cx, other_users_vec, is_group, name).await
}
