use math;
use math::{One, Rotation, Transform as Trans};
use ecs::{Entity, VecStorage, ArenaGetter};
use std::borrow::Borrow;

use super::errors::*;

/// `Transform` is used to store and manipulation the postiion, rotation and scale
/// of the object. Every `Transform` can have a parent, which allows you to apply
/// position, rotation and scale hierarchically.
///
/// `Entity` are used to record the tree relationships. Every access requires going
/// through the arena, which can be cumbersome and comes with some runtime overhead.
/// But it not only keeps code clean and simple, but also makes `Transform` could be
/// send or shared across threads safely. This enables e.g. parallel tree traversals.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
    parent: Option<Entity>,
    next_sib: Option<Entity>,
    prev_sib: Option<Entity>,
    first_child: Option<Entity>,
}

/// Declare `Transform` as component with compact vec storage.
declare_component!(Transform, VecStorage);

impl Default for Transform {
    fn default() -> Self {
        Transform {
            decomposed: math::Decomposed::one(),
            parent: None,
            next_sib: None,
            prev_sib: None,
            first_child: None,
        }
    }
}

impl Transform {
    #[inline]
    pub fn scale(&self) -> f32 {
        self.decomposed.scale
    }

    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        self.decomposed.scale = scale;
    }

    #[inline]
    pub fn position(&self) -> math::Vector3<f32> {
        self.decomposed.disp
    }

    #[inline]
    pub fn set_position<T>(&mut self, position: T)
        where T: Borrow<math::Vector3<f32>>
    {
        self.decomposed.disp = *position.borrow();
    }

    #[inline]
    pub fn translate<T>(&mut self, disp: T)
        where T: Borrow<math::Vector3<f32>>
    {
        self.decomposed.disp += *disp.borrow();
    }

    #[inline]
    pub fn rotation(&self) -> math::Quaternion<f32> {
        self.decomposed.rot
    }

    #[inline]
    pub fn set_rotation<T>(&mut self, rotation: T)
        where T: Borrow<math::Quaternion<f32>>
    {
        self.decomposed.rot = *rotation.borrow();
    }

    #[inline]
    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    // Return ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.first_child.is_none()
    }

    // Return ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

impl Transform {
    /// Attach a new child to parent transform, before existing children.
    ///
    /// If `keep_world_pose` is true, the parent-relative position, scale and rotation is
    /// modified such that the object keeps the same world space position, rotation and
    /// scale as before.
    pub fn set_parent(mut arena: &mut ArenaGetter<Transform>,
                      child: Entity,
                      parent: Option<Entity>,
                      keep_world_pose: bool)
                      -> Result<()> {
        unsafe {
            if arena.get(*child).is_none() {
                bail!(ErrorKind::NonTransformFound);
            }

            // Can not append a transform to it self.
            if let Some(parent) = parent {
                if parent == child || arena.get(*parent).is_none() {
                    bail!(ErrorKind::CanNotAttachSelfAsParent);
                }
            }

            // Retrive pose in world space.
            let decomposed = Transform::world_decomposed(&arena, child);

            Transform::remove_from_parent(arena, child)?;
            if let Some(parent) = parent {
                let next_sib = {
                    let node = arena.get_unchecked_mut(*parent);
                    ::std::mem::replace(&mut node.first_child, Some(child))
                };

                let child = arena.get_unchecked_mut(*child);
                child.parent = Some(parent);
                child.next_sib = next_sib;
            }

            // Revert to world pose.
            if keep_world_pose {
                Transform::set_world_decomposed(&mut arena, child, &decomposed)?;
            }

            Ok(())
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(arena: &mut ArenaGetter<Transform>, handle: Entity) -> Result<()> {
        unsafe {
            let (parent, next_sib, prev_sib) = {
                if let Some(node) = arena.get_mut(*handle) {
                    (node.parent.take(), node.next_sib.take(), node.prev_sib.take())
                } else {
                    bail!(ErrorKind::NonTransformFound);
                }
            };

            if let Some(next_sib) = next_sib {
                arena.get_unchecked_mut(*next_sib).prev_sib = prev_sib;
            }

            if let Some(prev_sib) = prev_sib {
                arena.get_unchecked_mut(*prev_sib).next_sib = next_sib;
            } else if let Some(parent) = parent {
                // Take this transform as the first child of parent if there is no previous sibling.
                arena.get_unchecked_mut(*parent).first_child = next_sib;
            }

            Ok(())
        }
    }

    /// Return an iterator of references to its ancestors.
    pub fn ancestors<'a, 'b>(arena: &'a ArenaGetter<'b, Transform>,
                             handle: Entity)
                             -> Ancestors<'a, 'b>
        where 'a: 'b
    {
        Ancestors {
            arena: &arena,
            cursor: arena.get(*handle).and_then(|v| v.parent),
        }
    }

    /// Returns an iterator of references to this transform's children.
    pub fn children<'a, 'b>(arena: &'a ArenaGetter<'b, Transform>,
                            handle: Entity)
                            -> Children<'a, 'b>
        where 'a: 'b
    {
        Children {
            arena: &arena,
            cursor: arena.get(*handle).and_then(|v| v.first_child),
        }
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    pub fn descendants<'a, 'b>(arena: &'a ArenaGetter<'b, Transform>,
                               handle: Entity)
                               -> Descendants<'a, 'b>
        where 'a: 'b
    {
        Descendants {
            arena: &arena,
            root: handle,
            cursor: arena.get(*handle).and_then(|v| v.first_child),
        }
    }

    /// Return true if rhs is one of the ancestor of this `Transform`.
    pub fn is_ancestor(arena: &ArenaGetter<Transform>, lhs: Entity, rhs: Entity) -> bool {
        for v in Transform::ancestors(arena, lhs) {
            if v == rhs {
                return true;
            }
        }

        false
    }

    /// Set the scale of `Transform` in world space.
    pub fn set_world_scale(arena: &mut ArenaGetter<Transform>,
                           handle: Entity,
                           world_scale: f32)
                           -> Result<()> {
        unsafe {
            if arena.get(*handle).is_some() {
                let mut scale = 1.0;
                for v in Transform::ancestors(arena, handle) {
                    scale *= arena.get_unchecked(*v).scale();
                }

                if !ulps_eq!(scale, 0.0) {
                    arena.get_unchecked_mut(*handle).set_scale(world_scale / scale);
                    Ok(())
                } else {
                    bail!(ErrorKind::CanNotInverseTransform);
                }
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Get the scale of `Transform` in world space.
    pub fn world_scale(arena: &ArenaGetter<Transform>, handle: Entity) -> Result<f32> {
        unsafe {
            if let Some(transform) = arena.get(*handle) {
                let mut scale = transform.scale();
                for v in Transform::ancestors(arena, handle) {
                    scale *= arena.get_unchecked(*v).scale();
                }
                Ok(scale)
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// set the position of `transform` in world space.
    pub fn set_world_position<T>(arena: &mut ArenaGetter<Transform>,
                                 handle: Entity,
                                 disp: T)
                                 -> Result<()>
        where T: Borrow<math::Vector3<f32>>
    {
        unsafe {
            if arena.get(*handle).is_some() {
                let decomposed = Transform::world_decomposed(&arena, handle);
                let delta = *disp.borrow() - decomposed.disp;
                arena.get_unchecked_mut(*handle).decomposed.disp = delta;
                Ok(())
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Get the position of `Transform` in world space.
    pub fn world_position(arena: &ArenaGetter<Transform>,
                          handle: Entity)
                          -> Result<math::Vector3<f32>> {
        unsafe {
            if arena.get(*handle).is_some() {
                Ok(Transform::world_decomposed(arena, handle).disp)
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Set the rotation of `Transform` in world space.
    pub fn set_world_rotation<T>(arena: &mut ArenaGetter<Transform>,
                                 handle: Entity,
                                 world_rotation: T)
                                 -> Result<()>
        where T: Borrow<math::Quaternion<f32>>
    {
        unsafe {
            if arena.get(*handle).is_some() {
                let mut rotation = math::Quaternion::one();
                for v in Transform::ancestors(arena, handle) {
                    rotation = rotation * arena.get_unchecked(*v).rotation();
                }

                let rotation = *world_rotation.borrow() * rotation.invert();
                arena.get_unchecked_mut(*handle).set_rotation(rotation);
                Ok(())
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Get the rotation of `Transform` in world space.
    pub fn world_rotation(arena: &ArenaGetter<Transform>,
                          handle: Entity)
                          -> Result<math::Quaternion<f32>> {
        unsafe {
            if let Some(transform) = arena.get(*handle) {
                let mut rotation = transform.rotation();
                for v in Transform::ancestors(arena, handle) {
                    rotation = rotation * arena.get_unchecked(*v).rotation();
                }
                Ok(rotation)
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    pub fn transform_vector(arena: &ArenaGetter<Transform>,
                            handle: Entity,
                            vec: math::Vector3<f32>)
                            -> Result<math::Vector3<f32>> {
        unsafe {
            if arena.get(*handle).is_some() {
                let decomposed = Transform::world_decomposed(arena, handle);
                Ok(decomposed.transform_vector(vec))
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Transforms position from local space to world space.
    pub fn transform_point(arena: &ArenaGetter<Transform>,
                           handle: Entity,
                           vec: math::Vector3<f32>)
                           -> Result<math::Vector3<f32>> {
        unsafe {
            if arena.get(*handle).is_some() {
                let decomposed = Transform::world_decomposed(&arena, handle);
                Ok(decomposed.rot * (vec * decomposed.scale) + decomposed.disp)
            } else {
                bail!(ErrorKind::NonTransformFound);
            }
        }
    }

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    pub fn transform_direction(arena: &ArenaGetter<Transform>,
                               handle: Entity,
                               vec: math::Vector3<f32>)
                               -> Result<math::Vector3<f32>> {
        if arena.get(*handle).is_some() {
            let rotation = Transform::world_rotation(&arena, handle)?;
            Ok(rotation * vec)
        } else {
            bail!(ErrorKind::NonTransformFound);
        }
    }

    /// Return the up direction in world space.
    pub fn up(arena: &ArenaGetter<Transform>, handle: Entity) -> Result<math::Vector3<f32>> {
        Transform::transform_direction(&arena, handle, math::Vector3::new(0.0, 1.0, 0.0))
    }

    /// Return the forward direction in world space.
    pub fn forward(arena: &ArenaGetter<Transform>, handle: Entity) -> Result<math::Vector3<f32>> {
        Transform::transform_direction(&arena, handle, math::Vector3::new(0.0, 0.0, 1.0))
    }

    unsafe fn set_world_decomposed(arena: &mut ArenaGetter<Transform>,
                                   handle: Entity,
                                   decomposed: &math::Decomposed<math::Vector3<f32>,
                                                                 math::Quaternion<f32>>)
                                   -> Result<()> {
        let mut relative = math::Decomposed::one();
        for v in Transform::ancestors(arena, handle) {
            relative = relative.concat(&arena.get_unchecked(*v).decomposed);
        }

        if let Some(inverse) = relative.inverse_transform() {
            arena.get_unchecked_mut(*handle).decomposed = decomposed.concat(&inverse);
            Ok(())
        } else {
            bail!(ErrorKind::CanNotInverseTransform);
        }
    }

    unsafe fn world_decomposed(arena: &ArenaGetter<Transform>,
                               handle: Entity)
                               -> math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>> {
        let mut decomposed = arena.get_unchecked(*handle).decomposed;
        for v in Transform::ancestors(arena, handle) {
            decomposed = decomposed.concat(&arena.get_unchecked(*v).decomposed);
        }
        decomposed
    }
}

pub struct Ancestors<'a, 'b>
    where 'a: 'b
{
    arena: &'b ArenaGetter<'a, Transform>,
    cursor: Option<Entity>,
}

impl<'a, 'b> Iterator for Ancestors<'a, 'b>
    where 'a: 'b
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let v = &self.arena.get_unchecked(*node);
                return ::std::mem::replace(&mut self.cursor, v.parent);
            }

            None
        }
    }
}

/// An iterator of references to its children.
pub struct Children<'a, 'b>
    where 'a: 'b
{
    arena: &'b ArenaGetter<'a, Transform>,
    cursor: Option<Entity>,
}

impl<'a, 'b> Iterator for Children<'a, 'b>
    where 'a: 'b
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let v = &self.arena.get_unchecked(*node);
                return ::std::mem::replace(&mut self.cursor, v.next_sib);
            }

            None
        }
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct Descendants<'a, 'b>
    where 'a: 'b
{
    arena: &'b ArenaGetter<'a, Transform>,
    root: Entity,
    cursor: Option<Entity>,
}

impl<'a, 'b> Iterator for Descendants<'a, 'b>
    where 'a: 'b
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let mut v = self.arena.get_unchecked(*node);

                // Deep first search when iterating children recursively.
                if v.first_child.is_some() {
                    return ::std::mem::replace(&mut self.cursor, v.first_child);
                }

                if v.next_sib.is_some() {
                    return ::std::mem::replace(&mut self.cursor, v.next_sib);
                }

                // Travel back when we reach leaf-node.
                while let Some(parent) = v.parent {
                    if parent == self.root {
                        break;
                    }

                    v = self.arena.get_unchecked(*v.parent.unwrap());
                    if v.next_sib.is_some() {
                        return ::std::mem::replace(&mut self.cursor, v.next_sib);
                    }
                }
            }

            return ::std::mem::replace(&mut self.cursor, None);
        }
    }
}