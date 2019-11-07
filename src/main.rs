#![allow(unused_imports, dead_code, unused_variables, unused_must_use)]

use market_sim1;
use arrayvec::ArrayVec;
use market_sim1::goods::{Task, Good::{Food, Grain}, Good};
use market_sim1::agent::Agent;
use std::collections::{HashMap, HashSet};
use maplit::hashmap;
use market_sim1::market::{Market, LinearMarket, SupplyPrice};
use std::iter::FromIterator;
use std::io;
use market_sim1::record::{init_recorder, add, register, flush, set_tick};
use std::cell::{RefCell, Cell};
use std::fs::File;
use rand::SeedableRng;
use rand::prelude::SmallRng;

fn main() {
    init_recorder("test", false);
    let tasks = vec![
        Task::new("Bake", &[(Grain, 30)], (Food, 10)),
        Task::new("Farm", &[], (Grain, 10)),
    ];

    let agents = {
        let mut agents = Vec::with_capacity(5);
        for i in 0..10 {
            agents.push(Agent::new(100., hashmap! {Grain => 10, Food => 10}));
        }
        agents
    };

    let market = LinearMarket::new(hashmap! {
        Grain => SupplyPrice::new(400, 10, -0.01),
        Food => SupplyPrice::new(400, 30, -0.01)
    });

    register("deaths", &["agent_id"]);
    register("tasks", &["task_name", "task_value", "revenue", "cost", "agent_id"]);
    register("price", &["good", "price", "supply"]);
    register("agent_info", &["agent_id", "cash", "food", "grain"]);
    register("food_consump", &["agent_id", "consump"]);

    run(tasks, agents, market, 20);

    flush();
}

fn run(tasks: Vec<Task>,
       mut agents: Vec<Agent>,
       mut market: impl Market,
       max_iters: u16) {
    let mut dead = HashSet::with_capacity(100);
    let rng = SmallRng::from_entropy();

    for i in 0..max_iters {
        set_tick(i);
        println!("{}", i);
        add("price", ("Food", market.price(Food, 1), market.supply(Food)));
        add("price", ("Grain", market.price(Grain, 1), market.supply(Grain)));

        const FOOD_UTILS: [i16; 9] = [60, 35, 33, 32, 30, 28, 10, 5, 3];
        // agents consume food
        for a in &mut agents {
            let mut consump = 0;
            for (j, &util) in FOOD_UTILS.iter().enumerate() {
                let val = market.value(Food, 1);
                if (util as f64) < val || (j > 1 && a.cash < val * 2.) {
                    break;
                }
                let new_food = a.res[&Food] - 1;
                if new_food < 0 {
                    if let Err(e) = market.buy(a, Food, 1) {
                        if j < 1 {
                            println!("death!, {}", a.id);
                            dead.insert(a.id);
                        }
                        break;
                    }
                }
                *a.res.get_mut(&Food).unwrap() -= 1;
                consump = j;
            }

            add("food_consump", (a.id, consump));
        }

        // remove dead agents
        for &a in &dead {
            agents.retain(|x| x.id != a);
            add("deaths", a)
        }
        dead.clear();


        // agents choose what to produce and produce it
        for a in agents.iter_mut() {
            let task = a.choose_task(&tasks, &market);
            add("tasks", (&task.name, task.value(&market), a.id));
            a.perform_task(task, &mut market);
        }

        // sell everything
        for a in agents.iter_mut() {
            let v: Vec<_> = a.res.iter()
                .map(|(&g, &a)| (g.clone(), a.clone()))
                .collect();
            for (g, amt) in v {
                if market.price(g, amt) > 0.5 {
                    market.sell(a, g, amt);
                }
            }
        }

        for a in &agents {
            add("agent_info", (a.id, &a.cash, &a.res[&Food], &a.res[&Grain]))
        }
    }
}
