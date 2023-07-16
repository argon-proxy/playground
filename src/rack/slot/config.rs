use std::marker::PhantomData;

use super::TunRackSlot;
use crate::rack::TunRackSlotRunner;

pub struct TunRackSlotConfig<S: TunRackSlot, R: TunRackSlotRunner<S>> {
    pub runner: PhantomData<R>,
    pub slot: PhantomData<S>,
}

impl<S: TunRackSlot, R: TunRackSlotRunner<S>> TunRackSlotConfig<S, R> {
    pub fn configure(&mut self, slot: S) -> R {
        // here, do whatever config is necessary
        R::new(slot)
    }
}
