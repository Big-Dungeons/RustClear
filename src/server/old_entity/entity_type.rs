use std::fmt::Formatter;

/// if this becomes big, we should use a hashset.
pub const NON_LIVING: [EntityType; 1] = [
    EntityType::Player,
];

crate::entity_type_registry! {
    Zombie: zombie,
    Player: player,
}

/// macro to register entity types. this should handle all the unique entity type functions and whatnot.
/// only has 2 right now, and nothing which may need parameters, but it should be supported.
/// you can have optional ones but clarifying that theyre optional in the macro itself is weird,
/// so atm they should be implemented but returning none.
///
/// structure:
/// ```
/// Name: implpath
/// ```
#[macro_export]
macro_rules! entity_type_registry {
    {$($name:ident: $path:ident),* $(,)?} => {
        #[derive(Clone, Debug, PartialEq, Eq, Copy)]
        pub enum EntityType {
            $(
                $name,
            )*
        }

        impl EntityType {
            pub const fn get_id(&self) -> i8 {
                match self {
                    $(
                        Self::$name => crate::server::old_entity::r#impl::$path::ID
                    ),*
                }
            }

            pub fn get_tasks(&self) -> Option<crate::server::old_entity::ai::ai_tasks::AiTasks> {
                match self {
                    $(
                        Self::$name => crate::server::old_entity::r#impl::$path::ai_tasks()
                    ),*
                }
            }

            pub fn metadata(&self) -> crate::server::old_entity::metadata::EntityMetadata {
                match self {
                    $(
                        Self::$name => crate::server::old_entity::r#impl::$path::metadata()
                    ),*
                }
            }
            
            pub const fn get_width(&self) -> f32 {
                match self {
                    $(
                        Self::$name => crate::server::old_entity::r#impl::$path::WIDTH
                    ),*
                }
            }
            pub const fn get_height(&self) -> f32 {
                match self {
                    $(
                        Self::$name => crate::server::old_entity::r#impl::$path::HEIGHT
                    ),*
                }
            }
        }
        
        impl std::fmt::Display for EntityType {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$name => write!(f, stringify!($name)),
                    )*
                }
            }
        }
    };
}