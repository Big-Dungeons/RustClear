use crate::{Player, PlayerExtension};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct PlayerIterator<'a, I, P : PlayerExtension>
where
    I: Iterator<Item = Rc<UnsafeCell<Player<P>>>>,
{
    iter: I,
    _marker: PhantomData<&'a mut P>,
}

impl<'a, I, P : PlayerExtension> PlayerIterator<'a, I, P>
where
    I: Iterator<Item = Rc<UnsafeCell<Player<P>>>>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            _marker: PhantomData,
        }
    }
}

impl<'a, I, P : PlayerExtension> Iterator for PlayerIterator<'a, I, P>
where
    I: Iterator<Item = Rc<UnsafeCell<Player<P>>>>,
{
    type Item = &'a mut Player<P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|rc| unsafe { &mut *rc.get() })
    }
}