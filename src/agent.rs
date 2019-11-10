use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};

use maplit::hashmap;

use crate::goods::{Good, Task};
use crate::goods::Good::{Food, Grain};
use crate::market::Market;

pub type AgentId = u16;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Agent {
    pub id: AgentId,
    pub cash: i16,
    pub res: HashMap<Good, i16>,
}

// track last used id
static ID: AtomicU16 = AtomicU16::new(0);

pub fn new_agent_id() -> u16 {
    ID.fetch_add(1, Relaxed)
}

#[derive(Debug)]
pub struct MU(Vec<i16>);


impl Agent {
    pub fn choose_trade(&self, price: i16, mu: &MU, good: Good) -> i16 {
        let p= price;
        let supply = self.res[&good];

        // find min to_trade s.t. the marginal utility of buying one more is less than the price
        let mut to_trade = 0;
        while mu.mu_buy(supply + to_trade) > p {
            to_trade += 1;
        }
        // find max to_trade s.t. the marginal utility of selling one more is greater than the price
        while mu.mu_sell(supply + to_trade) < p {
            to_trade -= 1;
        }
        to_trade
    }

    pub fn choose_task(&self, tasks: &'a [Task], market: &dyn Market) -> &'a Task {
        tasks.iter()
            .max_by_key(|&task| {
                let (val, _, cost) = task.value(market);
                if cost < self.cash { val as i32 } else { 0 }
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

    pub fn pre_made(num: usize) -> HashMap<AgentId, Agent> {
        let mut agents = HashMap::with_capacity(num);
        for _i in 0..num {
            Agent::new_into_map(&mut agents, 100, hashmap! {Grain => 10, Food => 10});
        }
        agents
    }

    pub fn new(cash: i16, res: HashMap<Good, i16>) -> Agent {
        Agent { id: new_agent_id(), cash, res }
    }

    pub fn new_into_map(map: &mut HashMap<u16, Agent>, cash: i16, res: HashMap<Good, i16>) {
        let id = new_agent_id();
        map.insert(id, Agent { id, cash, res });
    }

    pub fn new_with_id(id: u16, cash: i16, res: HashMap<Good, i16>) -> Agent {
        Agent { id, cash, res }
    }
}

impl MU {
    fn from_utility(u: &[i16]) -> MU {
        let mut mu = Vec::with_capacity(u.len() - 1);
        for i in 0..(u.len() - 1) {
            mu.push(u[i + 1] - u[i]);
        }
        MU(mu)
    }

    fn mu_buy(&self, supply: i16) -> i16 {
        self.0[supply as usize]
    }

    fn mu_sell(&self, supply: i16) -> i16 {
        assert!(supply > 0);
        self.0[(supply - 1) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mu() -> MU {
        let utility = [20_i16, 35, 47, 57, 62];
        MU::from_utility(&utility)
    }

    fn choose_trade_builder(p: i16, s: i16) -> i16 {
        let mut mu = make_mu();
        let a = Agent::new(20, hashmap!{Grain => s, Food => 40});
        a.choose_trade(p, &mu, Grain)
    }

    #[test]
    fn test_mu() {
        let mu = make_mu();

        assert_eq!(mu.mu_buy(2), 10);
        assert_eq!(mu.mu_sell(2), 12);
    }

    #[test]
    fn test_choose_trade1() {
        assert_eq!(choose_trade_builder(6, 2), 1);
        assert_eq!(choose_trade_builder(13, 2), -1);
        assert_eq!(choose_trade_builder(12, 2), 0);
        assert_eq!(choose_trade_builder(12, 0), 1);
        assert_eq!(choose_trade_builder(13, 0), 1);
        assert_eq!(choose_trade_builder(11, 0), 2);
    }
}