use arrayvec::ArrayVec;
use serde::Serialize;

use crate::market::Market;

#[derive(Copy, Hash, Clone, Eq, PartialOrd, PartialEq, Ord, Debug, Serialize)]
pub enum Good {
    Food,
    Grain,
}

impl Good {
    pub const ALL:[Good;2] = [Good::Food, Good::Grain];
}


#[derive(Clone, Eq, PartialOrd, PartialEq, Ord, Debug, Serialize)]
pub struct Task {
    pub inputs: ArrayVec<[(Good, i16); 4]>,
    pub output: (Good, i16),
    pub name: String,
}

impl Task {
    pub fn value(&self, market: &dyn Market, skill: f32) -> (i16, i16, i16) {
        let cost = self.inputs.iter()
            .map(|(good, amt)| market.value(*good, *amt))
            .sum();
        let revenue = market.value(self.output.0, (self.output.1 as f32 * skill).round() as i16);
        (revenue - cost, revenue, cost)
    }

    pub fn new(name: impl Into<String>, inputs: &[(Good, i16)], output: (Good, i16)) -> Task {
        let mut a = ArrayVec::new();
        a.try_extend_from_slice(inputs).unwrap();
        Task {
            name: name.into(),
            inputs: a,
            output,
        }
    }
}
