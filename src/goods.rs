use arrayvec::ArrayVec;

#[derive(Copy, Hash, Clone, Eq, PartialOrd, PartialEq, Ord, Debug)]
pub enum Good {
    Food,
    Grain,
}

#[derive(Clone, Eq, PartialOrd, PartialEq, Ord, Debug)]
pub struct Task {
    pub inputs: ArrayVec<[(Good, u16); 4]>,
    pub output: (Good, u16),
}


