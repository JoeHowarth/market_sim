use crate::goods::Good;
use crate::agent::{Agent, AgentId};
use failure::Error;
use std::sync::atomic::Ordering::AcqRel;
use std::collections::HashMap;
use crate::record::add;
use std::collections::hash_map::RandomState;
use rand::distributions::weighted::alias_method::WeightedIndex;
use std::iter::repeat;
use rand::prelude::{IteratorRandom, SmallRng, SliceRandom};
use rand::SeedableRng;
use crate::market::UnexecutedTrades::{Sells, Buys};

pub type GoodMap<T> = HashMap<Good, T>;

pub trait Market {
    fn price(&self, good: Good) -> i16;

    fn trade(&mut self, agent: &Agent, good: Good, amt: i16) -> Result<(), Error>;

    fn execute_trade(&mut self, agents: &mut HashMap<AgentId, Agent>, good: Good) -> UnexecutedTrades;

//    fn execute_trades(&mut self, agents: &mut [Agent]) -> UnexecutedTrades;

    fn update_price(&mut self, ts: UnexecutedTrades) -> Result<GoodMap<i16>, Error>;

    fn value(&self, good: Good, amt: i16) -> i16 {
        self.price(good) * amt
    }

    fn buy(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade(agent, good, amt)
    }

    fn sell(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade(agent, good, -amt)
    }
}

pub enum UnexecutedTrades {
    Buys(u16, u16),
    Sells(u16, u16),
}


pub struct ClearingMarket {
    pub prices: GoodMap<i16>,
    pub trades: GoodMap<Vec<(AgentId, i16)>>,
}

impl ClearingMarket {
    pub fn new(prices: GoodMap<i16>) -> ClearingMarket {
        let trades = prices.iter().map(|(&k,v)| (k, Vec::new())).collect();
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

impl Market for ClearingMarket {
    fn price(&self, good: Good) -> i16 {
        self.prices[&good]
    }

    fn trade(&mut self, agent: &Agent, good: Good, amt: i16) -> Result<(), Error> {
        if amt > 0 && agent.cash < self.value(good, amt) {
            Err(failure::err_msg("insufficient cash to make trade"))
        } else {
            Ok(self.trades.get_mut(&good).unwrap().push((agent.id, amt)))
        }
    }

    fn execute_trade(&mut self, agents: &mut HashMap<AgentId, Agent>, good: Good) -> UnexecutedTrades {
        let trades = self.trades
            .get_mut(&good)
            .unwrap();
        let total_trades = trades.len();
        let (mut buys, mut sells) = partition_and_shuffle_trades(trades);

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

        assert_eq!(num_trades, buys.len().max(sells.len()));
        match (buys.len(), sells.len()) {
            (0, x) => Sells(x as u16, total_trades as u16),
            (x, 0) => Buys(x as u16, total_trades as u16),
            (x, y) => {
                eprintln!("Shouldn't happen {}, {}", x, y);
                Sells(0, 0)
            }
        }
    }

    fn update_price(&mut self, _ts: UnexecutedTrades) -> Result<HashMap<Good, i16, RandomState>, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goods::Good::{Food, Grain};
    use maplit::hashmap;

    #[test]
    fn hi() {
        let mut agents = Agent::pre_made(2);
        let mut market = ClearingMarket::new(hashmap! { Food => 20, Grain => 20, });
        let keys: Vec<_> = agents.keys().into_iter().collect();
        let b = *keys[0];
        let s = *keys[1];

        market.trade(&agents[&b], Food, 2);
        market.trade(&agents[&s], Food, -2);

        assert_eq!(market.trades[&Food], vec![(b, 2), (s, -2)]);


    }
}

