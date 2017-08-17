//! #### Entity Component System (ECS)
//! ECS is an architectural pattern that is widely used in game development. It follows
//! the _Composition_ over _Inheritance_ principle that allows greater flexibility in
//! defining entities where every object in a game's scene in an entity.
//!
//! `Entity` is one of the most fundamental terms in this system. Its basicly some kind
//! of unique identifier to the in-game object. Every `Entity` consists of one or more
//! `Component`s, which define the internal data and how it interacts with the world.
//!
//! Its also common that abstracts `Entity` as container of components, buts with UID
//! approach, we could save the state externaly, users could transfer `Entity` easily
//! without considering the data-ownerships. The real data storage can be shuffled around
//! in memory as needed;
//!
//! #### Data Orinted Design
//! Data-oriented design is a program optimization approach motivated by cache coherency.
//! The approach is to focus on the data layout, separating and sorting fields according
//! to when they are needed, and to think about transformations of data.
//!
//! Due to the composition nature of ECS, its highly compatible with DOD. But benefits
//! doesn't comes for free, there are some memory/performance tradeoff generally. We
//! addressed some data storage approaches in `ecs::component`, users could make their
//! own decision based on different purposes.

use super::utility::handle::Handle;

#[macro_use]
mod iterator;
#[macro_use]
pub mod component;
pub mod world;

pub use self::component::{Component, ComponentStorage, HashMapStorage, VecStorage};
pub use self::world::{World, ArenaGetter};

pub type Entity = Handle;

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use super::*;
    use rand::{Rng, SeedableRng, XorShiftRng};

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct Position {
        x: u32,
        y: u32,
    }

    #[derive(Debug, Clone, Default)]
    struct Reference {
        value: Arc<RwLock<usize>>,
    }

    impl Drop for Reference {
        fn drop(&mut self) {
            *self.value.write().unwrap() += 1;
        }
    }

    declare_component!(Position, VecStorage);
    declare_component!(Reference, HashMapStorage);

    #[test]
    fn basic() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        world.assign::<Position>(e1, Position { x: 1, y: 2 });
        assert!(world.has::<Position>(e1));

        {
            let p = world.fetch::<Position>(e1).unwrap();
            assert_eq!(*p, Position { x: 1, y: 2 });
        }

        {
            let mut p = world.fetch_mut::<Position>(e1).unwrap();
            p.x = 2;
            p.y = 5;
        }

        {
            let p = world.fetch::<Position>(e1).unwrap();
            assert_eq!(*p, Position { x: 2, y: 5 });
        }

        world.remove::<Position>(e1);
        assert!(!world.has::<Position>(e1));
        assert!(world.fetch::<Position>(e1).is_none());
    }

    #[test]
    fn free() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Reference>();

        let e1 = world.create();
        assert!(world.is_alive(e1));
        assert!(!world.has::<Position>(e1));
        assert!(world.fetch::<Position>(e1).is_none());

        world.assign::<Position>(e1, Position { x: 1, y: 2 });
        assert!(world.has::<Position>(e1));
        world.fetch::<Position>(e1).unwrap();

        world.free(e1);
        assert!(!world.is_alive(e1));
        assert!(!world.has::<Position>(e1));
        assert!(world.fetch::<Position>(e1).is_none());

        let mut entities = Vec::new();
        let rc = Arc::new(RwLock::new(0));
        for i in 0..10 {
            let e = world.create();
            let shadow = rc.clone();
            entities.push(e);

            world.assign::<Reference>(e, Reference { value: shadow });
            if i % 2 == 0 {
                world.assign::<Position>(e, Position { x: 1, y: 2 });
            }
        }

        assert_eq!(*rc.read().unwrap(), 0);
        for i in 0..10 {
            world.free(entities[i]);
            assert_eq!(*rc.read().unwrap(), i + 1);
        }
        assert_eq!(*rc.read().unwrap(), 10);
    }

    #[test]
    fn duplicated_assign() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        assert!(world.assign::<Position>(e1, Position { x: 1, y: 2 }) == None);
        assert!(world.assign::<Position>(e1, Position { x: 2, y: 4 }) ==
                Some(Position { x: 1, y: 2 }));

        assert!(*world.fetch::<Position>(e1).unwrap() == Position { x: 2, y: 4 })
    }

    #[test]
    fn random_allocate() {
        let mut generator = XorShiftRng::from_seed([0, 1, 2, 3]);
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Reference>();

        let mut v = vec![];
        for i in 3..10 {
            let p = generator.next_u32() % i + 1;
            let r = generator.next_u32() % i + 1;
            for j in 0..100 {
                if j % p == 0 {
                    let e = world.create();
                    world.assign::<Position>(e,
                                             Position {
                                                 x: e.index(),
                                                 y: e.version(),
                                             });
                    if j % r == 0 {
                        world.assign_with_default::<Reference>(e);
                    }
                    v.push(e);
                }
            }

            let size = v.len() / 2;
            for _ in 0..size {
                let len = v.len();
                world.free(v.swap_remove(generator.next_u32() as usize % len));
            }
        }

        for i in v {
            assert_eq!(*world.fetch::<Position>(i).unwrap(),
                       Position {
                           x: i.index(),
                           y: i.version(),
                       });
        }
    }

    #[test]
    fn iter_with() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Reference>();

        let mut v = vec![];
        for i in 0..100 {
            let e = world.create();

            if i % 2 == 0 {
                world.assign::<Position>(e,
                                         Position {
                                             x: e.index(),
                                             y: e.version(),
                                         });
            }

            if i % 3 == 0 {
                world.assign_with_default::<Reference>(e);
            }

            if i % 2 == 0 && i % 3 == 0 {
                v.push(e);
            }
        }

        {
            let (view, arenas) = world.view_with_2::<Position, Reference>();
            for e in view {
                let p = Position {
                    x: e.index(),
                    y: e.version(),
                };

                assert_eq!(*arenas.0.get(e).unwrap(), p);
            }
        }

        {
            let (view, mut arenas) = world.view_with_2::<Position, Reference>();
            for e in view {
                arenas.0.get_mut(e).unwrap().x += e.version();
                *arenas.1.get_mut(e).unwrap().value.write().unwrap() += 1;
            }
        }

        {
            let (view, arenas) = world.view_with_2::<Position, Reference>();
            let mut iterator = view.into_iter();
            for e in &v {
                let i = iterator.next().unwrap();
                let p = Position {
                    x: e.index() + e.version(),
                    y: e.version(),
                };

                assert_eq!(i, *e);
                assert_eq!(*arenas.0.get(e).unwrap(), p);
                assert_eq!(*arenas.1.get(e).unwrap().value.read().unwrap(), 1);
            }
        }
    }

    #[test]
    #[should_panic]
    fn invalid_view() {
        let mut world = World::new();
        world.register::<Position>();

        let _i1 = world.view_with::<Position>();
        world.view_with::<Position>();
    }

    #[test]
    fn builder() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Reference>();

        let e1 = world.build().with_default::<Position>().finish();
        assert!(world.has::<Position>(e1));
        assert!(!world.has::<Reference>(e1));
    }

    #[test]
    #[should_panic]
    fn invalid_fetch() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.build().with_default::<Position>().finish();

        let _p1 = world.fetch_mut::<Position>(e1);
        world.fetch::<Position>(e1);
    }

    #[test]
    #[should_panic]
    fn invalid_fetch_mut() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.build().with_default::<Position>().finish();

        let _p1 = world.fetch_mut::<Position>(e1);
        world.fetch_mut::<Position>(e1);
    }
}