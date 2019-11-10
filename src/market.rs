use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::iter::{FromIterator, repeat};
use std::sync::atomic::Ordering::AcqRel;

use failure::Error;
use linear_map::LinearMap;
use rand::distributions::weighted::alias_method::WeightedIndex;
use rand::prelude::{IteratorRandom, SliceRandom, SmallRng};
use rand::SeedableRng;

use crate::agent::{Agent, AgentId};
use crate::goods::Good;
use crate::market::UnexecutedTrades::{All, Buys, Sells};
use crate::record::add;

pub type GoodMap<T> = LinearMap<Good, T>;

pub trait Market {
    fn price(&self, good: Good) -> i16;

    fn trade(&mut self, cash_and_id: (i16, u16), good: Good, amt: i16) -> Result<(), Error>;

    fn execute_trade(&mut self, agents: &mut HashMap<AgentId, Agent>, good: Good) -> UnexecutedTrades;

    fn execute_trades(&mut self, agents: &mut HashMap<AgentId, Agent>) -> GoodMap<UnexecutedTrades> {
        Good::ALL.iter()
            .map(|&good| {
                let unexecuted = self.execute_trade(agents, good);
                self.update_price(unexecuted, good);
                (good, unexecuted)
            })
            .collect()
    }

    fn update_price(&mut self, ts: UnexecutedTrades, good: Good) -> i16;

    fn value(&self, good: Good, amt: i16) -> i16 {
        self.price(good) * amt
    }

    fn buy(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade((agent.cash, agent.id), good, amt)
    }

    fn sell(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade((agent.cash, agent.id), good, -amt)
    }
}


pub struct ClearingMarket {
    pub prices: GoodMap<i16>,
    pub trades: GoodMap<Vec<(AgentId, i16)>>,
}

impl ClearingMarket {
    pub fn new(mut prices: HashMap<Good, i16>) -> ClearingMarket {
        let trades = prices.iter().map(|(&k, _)| (k, Vec::new())).collect();
        let prices = LinearMap::from_iter(prices.drain());
        ClearingMarket { prices, trades }
    }

    fn execute_transaction(&self, agents: &mut HashMap<AgentId, Agent>, buyer: AgentId, seller: AgentId, good: Good) {
        let price = self.price(good);

        let buyer = agents.get_mut(&buyer).unwrap();
        buyer.cash -= price;
        *buyer.res.get_mut(&good).unwrap() += 1;

        let seller = agents.get_mut(&seller).unwrap();
        seller.cash += price;
        *seller.res.get_mut(&good).unwrap() -= 1;
    }
}

fn partition_and_shuffle_trades(trades: &mut Vec<(AgentId, i16)>) -> (Vec<AgentId>, Vec<AgentId>) {
    type Trades = Vec<(AgentId, i16)>;
    let mut rng = SmallRng::from_entropy();
    let f = |pred: fn(i16) -> bool| {
        trades.iter()
            .filter(|x| pred(x.1))
            .flat_map(|(a, x)| repeat(*a).take(x.abs() as usize))
            .collect::<Vec<_>>()
    };

    let mut buys: Vec<AgentId> = f(|x| x > 0);
    let mut sells: Vec<AgentId> = f(|x| x < 0);
    buys.shuffle(&mut rng);
    sells.shuffle(&mut rng);
    trades.clear();
    (buys, sells)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnexecutedTrades {
    Buys(i16, i16),
    Sells(i16, i16),
    All(i16),
}

impl Market for ClearingMarket {
    fn price(&self, good: Good) -> i16 {
        self.prices[&good]
    }

    fn trade(&mut self, (cash, id): (i16, u16), good: Good, amt: i16) -> Result<(), Error> {
        if amt > 0 && cash < self.value(good, amt) {
            Err(failure::err_msg("insufficient cash to make trade"))
        } else {
            Ok(self.trades.get_mut(&good).unwrap().push((id, amt)))
        }
    }

    fn execute_trade(&mut self, agents: &mut HashMap<AgentId, Agent>, good: Good) -> UnexecutedTrades {
        let trades = self.trades
            .get_mut(&good)
            .unwrap();
        let (mut buys, mut sells) = partition_and_shuffle_trades(trades);
//        let total_trades = buys.len() + sells.len();
        let (total_sells, total_buys) = (sells.len(), buys.len());

        let num_trades = buys.len().min(sells.len());
        for _ in 0..num_trades {
            let b = buys.pop();
            let s = sells.pop();
            match (b, s) {
                (Some(b), Some(s)) => {
                    self.execute_transaction(agents, b, s, good);
                }
//                (None, Some(s)) => sells.push(s),
//                (Some(b), None) => buys.push(b),
                _ => eprintln!("Should never happen")
            }
        }

        match (buys.len(), sells.len()) {
            (0, 0) => All(((total_buys + total_sells) / 2) as i16),
            (0, x) => Sells(x as i16, total_sells as i16),
            (x, 0) => Buys(x as i16, total_buys as i16),
            (x, y) => {
                eprintln!("Shouldn't happen {}, {}", x, y);
                Sells(0, 0)
            }
        }
    }

    fn update_price(&mut self, ts: UnexecutedTrades, good: Good) -> i16 {
        let p = self.price(good);
        let pf = p as f64;
        let p_new = match ts {
            All(_) => p,
            Buys(unexecuted, executed) => {
                (pf * (1. + 0.25 * unexecuted as f64 / executed as f64)).round() as i16
            }
            Sells(unexecuted, executed) => {
                (pf * (1. - 0.25 * unexecuted as f64 / executed as f64)).round() as i16
            }
        };
        dbg!(p_new, p, ts);
//        match ts {
//            All(_) => assert_eq!(p_new, p),
//            Buys(unexecuted, executed) => {
//                assert!(p_new > p)
//            }
//            Sells(unexecuted, executed) => {
//                assert!(p_new < p)
//            }
//        }
        self.prices.insert(good, p_new);
        p_new
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use crate::goods::Good::{Food, Grain};

    use super::*;

    #[test]
    fn hi() {
        let mut agents = Agent::pre_made(2);
        let mut market = ClearingMarket::new(hashmap! { Food => 20, Grain => 20, });
        let keys: Vec<_> = agents.keys().into_iter().collect();
        let b = *keys[0];
        let s = *keys[1];

        market.trade((agents[&b].cash, agents[&b].id), Food, 2).unwrap();
        market.trade((agents[&s].cash, agents[&s].id), Food, -2).unwrap();

        let b_f = agents[&b].res[&Food];
        let s_f = agents[&s].res[&Food];
        assert_eq!(market.trades[&Food], vec![(b, 2), (s, -2)]);

        let rem = market.execute_trade(&mut agents, Food);

        assert_eq!(rem, All(2));
        assert_eq!(agents[&b].res[&Food], b_f + 2);
        assert_eq!(agents[&s].res[&Food], s_f - 2);

        let p = market.price(Food);
        let p1 = market.update_price(rem, Food);
        assert_eq!(p1, p);
    }

    #[test]
    fn buy_heavy() {
        let mut agents = Agent::pre_made(3);
        let mut market = ClearingMarket::new(hashmap! { Food => 20, Grain => 20, });
        let keys: Vec<_> = agents.keys().into_iter().collect();
        let b = *keys[0];
        let b1 = *keys[1];
        let s = *keys[2];

        market.trade((agents[&b].cash, agents[&b].id), Food, 2).unwrap();
        market.trade((agents[&b1].cash, agents[&b1].id), Food, 2).unwrap();
        market.trade((agents[&s].cash, agents[&s].id), Food, -2).unwrap();

        let b_f = agents[&b].res[&Food];
        let b1_f = agents[&b1].res[&Food];
        let s_f = agents[&s].res[&Food];
        assert_eq!(market.trades[&Food], vec![(b, 2), (b1, 2), (s, -2)]);

        let rem = market.execute_trade(&mut agents, Food);

        assert_eq!(rem, Buys(2, 4));
        assert_eq!(agents[&b].res[&Food] + agents[&b1].res[&Food], b1_f + b_f + 2);
        assert_eq!(agents[&s].res[&Food], s_f - 2);

        let p = market.price(Food);
        let p1 = market.update_price(rem, Food);
        assert_eq!(p1, (p as f64 * 1.125).round() as i16);
    }
}

