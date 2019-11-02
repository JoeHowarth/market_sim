use std::collections::HashMap;
use crate::goods::Good;

#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    pub id: u16,
    pub cash: f64,
    pub res: HashMap<Good, u16>
}