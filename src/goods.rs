use arrayvec::ArrayVec;
use crate::market::Market;

#[derive(Copy, Hash, Clone, Eq, PartialOrd, PartialEq, Ord, Debug)]
pub enum Good {
    Food,
    Grain,
}

#[derive(Clone, Eq, PartialOrd, PartialEq, Ord, Debug)]
pub struct Task {
    pub inputs: ArrayVec<[(Good, i16); 4]>,
    pub output: (Good, i16),
    pub name: String,
}

impl Task {
    pub(crate) fn value(&self, market: &dyn Market) -> f64 {
        let cost: f64 = self.inputs.iter()
            .map(|(good, amt)| market.value(*good, *amt))
            .sum();
        let revenue = market.value(self.output.0, self.output.1);

        revenue - cost
    }

    pub fn new(name: impl Into<String>, inputs: &[(Good, i16)], output: (Good, i16)) -> Task {
        let mut a = ArrayVec::new();
        a.try_extend_from_slice(inputs).unwrap();
        Task {
            name: name.into(),
            inputs: a,
            output
        }
    }
}
