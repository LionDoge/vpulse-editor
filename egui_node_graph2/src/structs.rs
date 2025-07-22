#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct CircularStack<T>
{
    memory: Vec<T>,
    size: usize,
    top: usize, // element index to be inserted
    fullness: usize,
}

impl<T> Default for CircularStack<T> {
    fn default() -> Self {
        Self::new(50)
    }
}

impl<T> CircularStack<T> {
    fn new(size: usize) -> Self {
        Self {
            memory: Vec::with_capacity(size),
            size,
            top: 0,
            fullness: 0,
        }
    }

    fn push(&mut self, element: T) {
        self.memory[self.top] = element;
        self.top = (self.top + 1) % self.size;
        if self.fullness < self.size {
            self.fullness += 1;
        }
    }

    fn pop(&mut self) -> Option<&T> {
        if self.fullness == 0 {
            None
        } else {
            self.top = (self.top - 1) % self.size;
            let element = self.memory.get(self.top);
            assert!(element.is_some(), "Element should exist in the stack");
            self.fullness -= 1;
            element
        }
    }
}