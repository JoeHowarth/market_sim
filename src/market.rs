use crate::goods::Good;
use crate::agent::Agent;
use failure::Error;
use failure::_core::sync::atomic::Ordering::AcqRel;
use crate::market::AddSupplyResponse::InsufficientSupply;
use std::collections::HashMap;
use crate::record::add;

pub trait Market {
    fn price(&self, good: Good, amt: i16) -> f64;

    fn value(&self, good: Good, amt: i16) -> f64 {
        self.price(good, amt) * amt as f64
    }

    fn add_supply(&mut self, good: Good, amt: i16) -> AddSupplyResponse;
    fn supply(&mut self, good: Good) -> i16;

    fn trade(&mut self, agent: &mut Agent, good: Good, amt: i16) -> Result<(), Error> {
        let value = self.value(good, amt);

        if agent.cash < value {
            return Err(failure::err_msg("Insufficient cash"));
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

pub trait IndPrice {
    fn price(&self, amt: i16) -> f64;
    fn add_supply(&mut self, amt: i16) -> AddSupplyResponse;
    fn supply(&self) -> i16;
}

pub struct SupplyPrice {
    pub supply: i16,
    s0: i16,
    p0: i16,
    slope: f64
}

impl SupplyPrice {
    pub fn new(s0: i16, p0: i16, slope: f64) -> SupplyPrice {
        SupplyPrice { supply: s0, s0, p0, slope }
    }
}

impl IndPrice for SupplyPrice {
    fn price(&self, amt: i16) -> f64 {
        let x = self.supply + amt / 2;
        self.slope * (x - self.s0) as f64 + self.p0 as f64
    }

    fn add_supply(&mut self, amt: i16) -> AddSupplyResponse {
        if self.supply + amt <= 0 {
            AddSupplyResponse::InsufficientSupply(self.supply, amt)
        } else {
            self.supply += amt;
            AddSupplyResponse::Ok(self.supply)
        }
    }

    fn supply(&self) -> i16 {
        self.supply
    }
}



pub struct IndMarket<T: IndPrice>(HashMap<Good, T>);
pub type LinearMarket = IndMarket<SupplyPrice>;

impl<T: IndPrice> IndMarket<T> {
    pub fn new(m: HashMap<Good, T>) -> Self {
        Self(m)
    }
}

impl<T: IndPrice> Market for IndMarket<T> {
    fn price(&self, good: Good, amt: i16) -> f64 {
        let p = self.0[&good].price(amt);
        add("price", (good, p, self.0[&good].supply()));
        p
    }

    fn add_supply(&mut self, good: Good, amt: i16) -> AddSupplyResponse {
        self.0.get_mut(&good).unwrap().add_supply(amt)
    }

    fn supply(&mut self, good: Good) -> i16 {
        self.0[&good].supply()
    }
}

#[derive(Fail, Debug)]
pub enum AddSupplyResponse {
    #[fail(display = "Insufficient supply: {}", _0)]
    InsufficientSupply(i16, i16),
    #[fail(display = "Update successful, new supply: {}", _0)]
    Ok(i16),
}