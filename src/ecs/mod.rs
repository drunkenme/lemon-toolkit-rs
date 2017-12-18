//! The entity component system with a data-orinted designs.
//!
//! # Entity Component System (ECS)
//!
//! ECS is an architectural pattern that is widely used in game development. It follows
//! the _Composition_ over _Inheritance_ principle that allows greater flexibility in
//! defining entities where every object in a game's scene in an entity.
//!
//! ## Entity
//!
//! `Entity` is one of the most fundamental terms in this system. Its basicly some kind
//! of unique identifier to the in-game object. Every `Entity` consists of one or more
//! `Component`s, which define the internal data and how it interacts with the world.
//!
//! Its also common that abstracts `Entity` as container of components, buts with UID
//! approach, we could save the state externaly, users could transfer `Entity` easily
//! without considering the data-ownerships. The real data storage can be shuffled around
//! in memory as needed.
//!
//! ## Data Orinted Design
//!
//! Data-oriented design is a program optimization approach motivated by cache coherency.
//! The approach is to focus on the data layout, separating and sorting fields according
//! to when they are needed, and to think about transformations of data.
//!
//! Due to the composition nature of ECS, its highly compatible with DOD. But benefits
//! doesn't comes for free, there are some memory/performance tradeoff generally. We
//! addressed some data storage approaches in `ecs::component`, users could make their
//! own decision based on different purposes.

use super::utils::handle::Handle;

mod bitset;
#[macro_use]
mod iterator;
#[macro_use]
pub mod component;
pub mod world;

pub use self::component::{Component, ComponentArena, HashMapArena, VecArena};
pub use self::world::{World, Arena, ArenaMut};

/// `Entity` type, as seen by the user, its a alias to `Handle` internally.
pub type Entity = Handle;