//! A simple observer with a single value.

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::{
    cell::{Ref, RefCell},
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    ops::Deref,
};

use ahash::RandomState;
use libafl_bolts::{ownedref::OwnedRef, Named};
use serde::{Deserialize, Serialize};

use super::Observer;
use crate::{inputs::UsesInput, observers::ObserverWithHashField, Error};

/// A simple observer with a single value.
///
/// The intent is that the value is something with interior mutability which the target could write to even though this
/// observer has a reference to it. Use [`RefCellValueObserver`] if using a [`RefCell`] around the value.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "T: serde::de::DeserializeOwned")]
pub struct ValueObserver<'a, T>
where
    T: Debug + Serialize,
{
    /// The name of this observer.
    name: String,
    /// The value.
    pub value: OwnedRef<'a, T>,
}

impl<'a, T> ValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    /// Creates a new [`ValueObserver`] with the given name.
    #[must_use]
    pub fn new(name: &'static str, value: &'a T) -> Self {
        Self {
            name: name.to_string(),
            value: OwnedRef::Ref(value),
        }
    }

    /// Get a reference to the underlying value.
    #[must_use]
    pub fn get_ref(&self) -> &T {
        self.value.as_ref()
    }

    /// Set the value.
    pub fn set(&mut self, new_value: T) {
        self.value = OwnedRef::Owned(Box::new(new_value));
    }

    /// Clone or move the current value out of this object.
    #[must_use]
    pub fn take(self) -> T
    where
        T: Clone,
    {
        match self.value {
            OwnedRef::Ref(r) => r.clone(),
            OwnedRef::Owned(v) => *v,
        }
    }
}

/// This *does not* reset the value inside the observer.
impl<'a, S, T> Observer<S> for ValueObserver<'a, T>
where
    S: UsesInput,
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, T> Named for ValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a, T: Hash> ObserverWithHashField for ValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn hash(&self) -> Option<u64> {
        let mut s = RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        Hash::hash(self.value.as_ref(), &mut s);
        Some(s.finish())
    }
}

/// A simple observer with a single [`RefCell`]'d value.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "T: serde::de::DeserializeOwned")]
pub struct RefCellValueObserver<'a, T>
where
    T: Debug + Serialize,
{
    /// The name of this observer.
    name: String,
    /// The value.
    pub value: OwnedRef<'a, RefCell<T>>,
}

impl<'a, T> RefCellValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    /// Creates a new [`RefCellValueObserver`] with the given name.
    #[must_use]
    pub fn new(name: &'static str, value: &'a RefCell<T>) -> Self {
        Self {
            name: name.to_string(),
            value: OwnedRef::Ref(value),
        }
    }

    /// Get a reference to the underlying value.
    #[must_use]
    pub fn get_ref<'b>(&'b self) -> Ref<'a, T>
    where
        'b: 'a,
    {
        self.value.as_ref().borrow()
    }

    /// Set the value.
    pub fn set(&mut self, new_value: T) {
        self.value.as_ref().replace(new_value);
    }

    /// Clone or move the current value out of this object.
    #[must_use]
    pub fn take(self) -> T
    where
        T: Clone,
    {
        match self.value {
            OwnedRef::Ref(r) => r.borrow().deref().clone(),
            OwnedRef::Owned(v) => v.borrow().clone(),
        }
    }
}

/// This *does not* reset the value inside the observer.
impl<'a, S, T> Observer<S> for RefCellValueObserver<'a, T>
where
    S: UsesInput,
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, T> Named for RefCellValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a, T: Hash> ObserverWithHashField for RefCellValueObserver<'a, T>
where
    T: Debug + Serialize + serde::de::DeserializeOwned,
{
    fn hash(&self) -> Option<u64> {
        let mut s = RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        self.value.as_ref().borrow().hash(&mut s);
        Some(s.finish())
    }
}
