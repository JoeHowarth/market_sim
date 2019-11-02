use crate::goods::Good;
use crate::agent::Agent;
use failure::Error;
use failure::_core::sync::atomic::Ordering::AcqRel;
use crate::market::AddSupplyResponse::InsufficientSupply;
use std::collections::HashMap;

pub trait Market {
    fn price(&self, good: Good, amt: i16) -> f64;

    fn value(&self, good: Good, amt: i16) -> f64 {
        self.price(good, amt) * amt as f64
    }

    fn add_supply(&mut self, good: Good, amt: i16) -> AddSupplyResponse;

    fn trade(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        let value = self.value(good, amt);

        if agent.cash < value {
            return Err(failure::err_msg("Insufficient cash"))
        }

        match self.add_supply(good, -amt) {
            AddSupplyResponse::Ok(_) => {
                agent.cash -= value;
                *agent.res.get_mut(&good).unwrap() += amt;
                Ok(())
            }
            x => Err(Error::from(x))
        }
    }

    fn buy(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade(agent, good, amt)
    }

    fn sell(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        self.trade(agent, good, -amt)
    }
}

struct SupplyPrice {
    pub supply: i16,
    pub price: fn(x: f64) -> f64,
}

pub struct LinearMarket(HashMap<Good, SupplyPrice>);

impl Market for LinearMarket {
    fn price(&self, good: Good, amt: i16) -> f64 {
        let SupplyPrice {supply, price} = self.0[&good];
        price(supply as f64 + amt as f64 / 2.)
    }

    fn add_supply(&mut self, good: Good, amt: i16) -> AddSupplyResponse {
        let sp= self.0.get_mut(&good).unwrap();
        if sp.supply + amt <= 0 {
            AddSupplyResponse::InsufficientSupply(sp.supply, amt)
        } else {
            sp.supply += amt;
            AddSupplyResponse::Ok(sp.supply)
        }
    }
}

#[derive(Fail, Debug)]
pub enum AddSupplyResponse {
    #[fail(display = "Insufficient supply: {}", _0)]
    InsufficientSupply(i16, i16),
    #[fail(display = "Update successful, new supply: {}", _0)]
    Ok(i16),
}