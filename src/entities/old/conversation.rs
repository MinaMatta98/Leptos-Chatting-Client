//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

#[cfg(feature = "ssr")]
pub mod server {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "conversation")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub last_message_at: DateTimeUtc,
        pub created_at: DateTimeUtc,
        pub name: String,
        pub is_group: i8,
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

    impl Related<crate::entities::users::server::Entity> for Entity {
        fn to() -> RelationDef {
            crate::entities::user_conversation::server::Relation::Users.def()
        }
        fn via() -> Option<RelationDef> {
            Some(
                crate::entities::user_conversation::server::Relation::Conversation
                    .def()
                    .rev(),
            )
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
