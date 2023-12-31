//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
    pub use super::conversation::server::Entity as Conversation;
    pub use super::message::server::Entity as Message;
    pub use super::seen_messages::server::Entity as SeenMessages;
    pub use super::temp_users::server::Entity as TempUsers;
    pub use super::user_conversation::server::Entity as UserConversation;
    pub use super::users::server::Entity as Users;
}
}
