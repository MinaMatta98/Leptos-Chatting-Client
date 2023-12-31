//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

#[cfg(feature = "ssr")]
pub mod server {
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: i64,
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::entities::message::server::Entity")]
    Message,
}

impl Related<crate::entities::message::server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl Related<crate::entities::conversation::server::Entity> for Entity {
    fn to() -> RelationDef {
        crate::entities::user_conversation::server::Relation::Conversation.def()
    }
    fn via() -> Option<RelationDef> {
        Some(crate::entities::user_conversation::server::Relation::Users.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
}
