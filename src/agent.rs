use std::collections::HashMap;
use crate::goods::{Good, Task};
use failure::_core::cell::Cell;
use crate::market::Market;

#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    pub id: u16,
    pub cash: f64,
    pub res: HashMap<Good, i16>
}

// track last used id
const ID: Cell<u16> = Cell::new(0);

impl Agent {
    pub fn new(cash: f64, res: HashMap<Good, i16>) -> Agent {
        ID.set(ID.get() + 1);
        Agent { id: ID.get(), cash, res }
    }

    pub fn choose_task(&self, tasks: &[&'a Task], market: &dyn Market) -> &'a Task {
        tasks.iter()
            .max_by_key(|&&task| task.value(market) as i32)
            .expect("If tasks non-empty, then should have best task")
    }
}