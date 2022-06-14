use alloc::vec::Vec;
use serde::{Serialize, Deserialize};
use crate::prelude::{CommandQueue, Event, Context, BaseEvent, EMPTY};
use super::{MemBuffer, MemFlag};

impl<T: Copy + Unpin + Serialize> MemBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn serialize_with_wait<S> (&self, serializer: S, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.serialize_with_queue(CommandQueue::default(), serializer, wait)
    }

    #[inline(always)]
    pub fn serialize_with_queue<S> (&self, queue: &CommandQueue, serializer: S, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let vec = self.to_vec_with_queue(queue, wait).map_err(|e| <S::Error as serde::ser::Error>::custom(e))?;
        let vec = vec.wait().map_err(|e| <S::Error as serde::ser::Error>::custom(e))?;
        vec.serialize(serializer)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub async fn serialize_async<S>(&self, serializer: S, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.serialize_with_queue_async(CommandQueue::default(), serializer, wait).await
    }

    #[inline(always)]
    pub async fn serialize_with_queue_async<S> (&self, queue: &CommandQueue, serializer: S, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let vec = self.to_vec_with_queue(queue, wait).map_err(|e| <S::Error as serde::ser::Error>::custom(e))?;
        let vec = vec.await.map_err(|e| <S::Error as serde::ser::Error>::custom(e))?;
        vec.serialize(serializer)
    }
}

#[cfg(feature = "def")]
impl<T: Copy + Unpin + Serialize> Serialize for MemBuffer<T> {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.serialize_with_wait(serializer, EMPTY)
    }
}

impl<'de, T: Copy + Unpin + Deserialize<'de>> MemBuffer<T> {
    #[inline(always)]
    pub fn deserialize_with_context<D> (ctx: &Context, deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Self::with_context(ctx, MemFlag::default(), &vec).map_err(|e| <D::Error as serde::de::Error>::custom(e))
    }
}

#[cfg(feature = "def")]
impl<'de, T: Copy + Unpin + Deserialize<'de>> Deserialize<'de> for MemBuffer<T> {
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Self::deserialize_with_context(Context::default(), deserializer)
    }
}