use std::collections::HashMap;
use crate::goods::{Good, Task};
use std::cell::Cell;
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use crate::market::Market;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Agent {
    pub id: u16,
    pub cash: f64,
    pub res: HashMap<Good, i16>
}

// track last used id
static ID: AtomicU16 = AtomicU16::new(0);

pub fn new_agent_id() -> u16 {
    ID.fetch_add(1, Relaxed)
}

impl Agent {
    pub fn new(cash: f64, res: HashMap<Good, i16>) -> Agent {
        Agent { id: new_agent_id(), cash, res }
    }

    pub fn new_into_map(map: &mut HashMap<u16, Agent>, cash: f64, res: HashMap<Good, i16>) {
        let id = new_agent_id();
        map.insert(id, Agent { id, cash, res });
    }

    pub fn new_with_id(id: u16, cash: f64, res: HashMap<Good, i16>) -> Agent {
        Agent { id, cash, res }
    }

    pub fn choose_task(&self, tasks: &'a [Task], market: &dyn Market) -> &'a Task {
        tasks.iter()
            .max_by_key(|&task| {
                let (val, _, cost) = task.value(market);
                if cost < self.cash {val as i32} else {0}
            })
            .expect("If tasks non-empty, then should have best task")
    }

    pub fn perform_task(&mut self, task: &Task, market: &mut dyn Market) {
        for &(good, amt) in &task.inputs {
            let owned: i16 = self.res[&good];
            if owned < amt {
                market.buy(self, good, (owned - amt).abs())
                    .unwrap();
            }
            *self.res.get_mut(&good).unwrap() -= amt;
        }
        let &(good, amt) = &task.output;
        *self.res.get_mut(&good).unwrap() += amt;
    }
}