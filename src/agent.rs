use std::cell::Cell;
use std::collections::HashMap;
use std::iter::repeat;
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};

use maplit::hashmap;
use rand::{Rng, SeedableRng};
use rand::prelude::SmallRng;

use crate::goods::{Good, Task};
use crate::goods::Good::{Food, Grain};
use crate::market::Market;
use std::cmp::Reverse;

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

#[derive(Debug, Clone)]
pub struct MU(pub Vec<(i16, u8)>);


impl Agent {
    pub fn choose_trade(&self, price: i16, mu: &MU, good: Good) -> i16 {
        let p = price;
        let supply = self.res[&good];

        dbg!(p, supply);

        // find min to_trade s.t. the marginal utility of buying one more is less than the price
        let mut to_trade = 0;
        while mu.mu_buy(supply + to_trade) > p {
            to_trade += 1;
        }
        // find max to_trade s.t. the marginal utility of selling one more is greater than the price
        while mu.mu_sell(supply + to_trade) < p && to_trade + supply >= 0 {
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
        let mut rng = SmallRng::from_entropy();
        for _i in 0..num {
            Agent::new_into_map(&mut agents,
                                100,
                                hashmap! {Grain => rng.gen_range(5, 30), Food => rng.gen_range(2, 15)});
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
    pub fn from_utility(u: &[i16], discount: f64) -> MU {
        let mut mu = Vec::with_capacity(u.len() - 1);
        let d2 = discount * discount;
        for i in 0..(u.len() - 1) {
            let cur = u[i + 1] - u[i];
            mu.push((cur, 0));
            mu.push(((cur as f64 * discount).round() as i16, 1));
            mu.push(((cur as f64 * d2).round() as i16, 2));
            mu.push(((cur as f64 * d2 * discount).round() as i16, 3));
            mu.push(((cur as f64 * d2 * d2).round() as i16, 4));
        }
        mu.sort_by_key(|&x| Reverse(x.clone()));
        MU(mu)
    }

    pub fn from_curr_mu(curr_mu: &[i16], discount: f64) -> MU {
        let mut mu = Vec::with_capacity(curr_mu.len() * 2);
        let d2 = discount * discount;
        for i in 0..curr_mu.len() {
            mu.push((curr_mu[i], 0));
            mu.push(((curr_mu[i] as f64 * discount).round() as i16, 1));
            mu.push(((curr_mu[i] as f64 * d2).round() as i16, 2));
            mu.push(((curr_mu[i] as f64 * d2 * discount).round() as i16, 3));
            mu.push(((curr_mu[i] as f64 * d2 * d2).round() as i16, 4));
        }
        mu.sort_by_key(|&x| Reverse(x.clone()));
        MU(mu)
    }

    pub fn from_market(market: &dyn Market, tasks: &[Task], good: Good) -> MU {
        let (mu, &input) = tasks.iter()
            .filter(|&t| t.inputs
                .iter()
                .any(|(g, _)| *g == good))
            .map(|task| {
                let (good_, input) = task.inputs
                    .iter()
                    .filter(|(g, _)| *g == good).take(1).next().unwrap();
                assert!(good == *good_);
                let (out_g, output) = task.output;
                let out_value = market.value(out_g, output);
                let mu_prime = out_value / input;
                dbg!((mu_prime, input))
            }).max_by_key(|(mu, _)| *mu).unwrap();
        dbg!(mu, input);

        MU((0..3)
            .flat_map(|i| repeat((dbg!((mu as f64 * dbg!(0.8_f64.powf(i as f64))) as i16), i))
                .take(input as usize))
            .collect())
    }

    pub fn utility(&self, u_0: i16) -> Vec<i16> {
        let mut util = Vec::with_capacity(self.0.len() + 2);
        util.push(u_0);
        for (i, &(d, _)) in self.0.iter().enumerate() {
            util.push(util[i] + d);
        }
        util
    }

    pub fn mu_consume(&self, supply: i16) -> i16 {
        let mut to_consume = 0;
        let mut to_save = 0;
        for (_d, i) in &self.0 {
            dbg!(to_consume, to_save, _d, i);
            if to_save + to_consume >= supply {
                break
            } else if *i > 0 {
                to_save += 1;
            } else {
                to_consume += 1;
            }
        }
        return to_consume;
    }

    fn mu_buy(&self, supply: i16) -> i16 {
        if supply as usize > self.0.len() - 1 {
            0
        } else {
            self.0[supply as usize].0
        }
    }

    fn mu_sell(&self, supply: i16) -> i16 {
        if supply as usize > self.0.len() - 1 || supply == 0 {
            0
        } else {
            self.0[supply as usize - 1].0
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::RandomState;

    use failure::Error;

    use crate::market::UnexecutedTrades;

    use super::*;

    #[test]
    fn test_from_market() {
        let tasks = vec![
            Task::new("Bake", &[(Grain, 30)], (Food, 10)),
            Task::new("Farm", &[], (Grain, 10)),
        ];
        let market = MockMarket(20);
        let mu = MU::from_market(&market, &tasks, Grain);

        assert_eq!(mu.mu_buy(1), 6);
        assert_eq!(mu.mu_buy(35), 4);
        assert_eq!(mu.mu_buy(1), mu.mu_buy(28));
    }

    #[test]
    fn test_mu() {
        let mu = make_mu();

        assert_eq!(mu.mu_buy(2), 10);
        assert_eq!(mu.mu_sell(2), 12);

        dbg!(&mu);
        assert_eq!(mu.mu_consume(3), 3);
        assert_eq!(mu.mu_consume(10), 4);
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

    fn make_mu() -> MU {
        let utility = [20_i16, 35, 47, 57, 62];
        MU::from_utility(&utility, 0.4)
    }

    fn choose_trade_builder(p: i16, s: i16) -> i16 {
        let mu = make_mu();
        let a = Agent::new(20, hashmap! {Grain => s, Food => 40});
        a.choose_trade(p, &mu, Grain)
    }

    struct MockMarket(pub i16);

    impl Market for MockMarket {
        fn price(&self, _good: Good) -> i16 {
            self.0
        }

        fn old_price(&self, _good: Good) -> i16 {
            unimplemented!()
        }

        fn trade(&mut self, _cash_and_id: (i16, u16), _good: Good, _amt: i16) -> Result<(), Error> {
            unimplemented!()
        }

        fn execute_trade(&mut self, _agents: &mut HashMap<u16, Agent, RandomState>, _good: Good) -> UnexecutedTrades {
            unimplemented!()
        }

        fn update_price(&mut self, _ts: UnexecutedTrades, _good: Good) -> i16 {
            unimplemented!()
        }
    }
}