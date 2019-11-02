#![allow(unused_imports, dead_code, unused_variables)]

use market_sim1;
use arrayvec::ArrayVec;
use market_sim1::goods::{Task, Good::{Food, Grain}};
use market_sim1::agent::Agent;
use std::collections::HashMap;
use maplit::hashmap;

fn main() {
    let tasks = [
        Task::new("Farm", &[(Grain, 30)], (Food, 10)),
        Task::new("Bake", &[], (Grain, 10)),
    ];

    let agents = [
        Agent::new(100., hashmap!{Grain => 10, Food => 10}),
        Agent::new(120., hashmap!(Grain => 20, Food => 5)),
    ];



//    let i: ArrayVec<[i32; 4]> = ArrayVec::from([1,2]);
}
