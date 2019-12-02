#![allow(unused_imports, dead_code, unused_variables, unused_must_use)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::iter::FromIterator;

use arrayvec::ArrayVec;
use maplit::hashmap;
use rand::prelude::SmallRng;
use rand::SeedableRng;

use market_sim1;
use market_sim1::agent::{Agent, AgentId, MU};
use market_sim1::goods::{Good::{Food, Grain}, Good, Task};
use market_sim1::market::{ClearingMarket, GoodMap, Market, UnexecutedTrades};
use market_sim1::record::{add, flush, init_recorder, register, set_tick};
use std::io::Write;

fn main() {
    dbg!("start");
    std::io::stdout().flush();
    init_recorder("adapt v1", true);
    let tasks = vec![
        Task::new("Bake", &[(Grain, 15)], (Food, 10)),
        Task::new("Farm", &[], (Grain, 10)),
    ];

    dbg!("here");
    let agents = Agent::pre_made(3);

    let market = ClearingMarket::new(hashmap! {
        Food => 25,
        Grain => 5,
    });
    dbg!("here2");

    register("deaths", &["agent_id"]);
    register("tasks", &["task_name", "task_value", "revenue", "cost", "agent_id"]);
    register("price", &["good", "new_price", "old_price", "unexecuted", "volume"]);
    register("agent_info", &["agent_id", "cash", "food", "grain"]);
    register("utility", &["agent_id", "utility", "food_consumed"]);

    dbg!("running...");
    run(tasks, agents, market, 20);

    flush();
}

fn run(tasks: Vec<Task>,
       mut agents: HashMap<AgentId, Agent>,
       mut market: impl Market,
       max_iters: u16) {
    let mut dead = HashSet::with_capacity(100);
    let rng = SmallRng::from_entropy();

    let food_mu = MU::from_curr_mu(&[120_i16, 60, 50, 40, 30, 20, 10, 2, 1], 0.8);
    let food_utils = food_mu.utility(0);

    for i in 0..max_iters {
        set_tick(i);
        println!("{}", i);
//        add("price", ("Food", market.price(Food) ));
//        add("price", ("Grain", market.price(Grain)));


        for trade_round in 0..2 {
            // register trades
            for &good in &Good::ALL {
                let price = market.price(good);
                let mu = match good {
                    Food => food_mu.clone(),
                    _ => MU::from_market(&market, &tasks, good)
                };
                for a in agents.values() {
                    let trade = a.choose_trade(price, &mu, good);
                    market.trade((a.cash, a.id), good, trade);
                }
            }
            let res = market.execute_trades(&mut agents);
            log_prices(&res, &market);
        }

        // consume food
        for a in agents.values_mut() {
            let food = a.res[&Food] as usize;
            if food <= 1 {
                dead.insert(a.id);
            }
            // they've already traded what they want, so eat it all!
            // later, factor in discounted consumption
            let consumption = 5.min(food_mu.mu_consume(food as i16));
            add("utility", (a.id, food_utils[food.min(5)], consumption));
            dbg!(food);
            *a.res.get_mut(&Food).unwrap() -= consumption;
            dbg!(a.res[&Food]);
        }

        // remove dead agents
        for a in &dead {
            agents.remove(a);
            add("deaths", a)
        }
        dead.clear();

        // agents choose what to produce and produce it
        for a in agents.values_mut() {
            let task = a.choose_task(&tasks, &market);
            add("tasks", (&task.name, task.value(&market), a.id));
            a.perform_task(task, &mut market);
            println!("Id {} working {:?}", a.id, &task.name);
        }

        for a in agents.values_mut() {
            add("agent_info", (a.id, &a.cash, &a.res[&Food], &a.res[&Grain]))
        }
    }
}

fn log_prices(res: &GoodMap<UnexecutedTrades>, market: &dyn Market) {
    for (&good, &t) in res {
        let (un, vol) = match t {
            UnexecutedTrades::Sells(un, vol) => (-un, vol),
            UnexecutedTrades::Buys(un, vol) => (un, vol),
            UnexecutedTrades::All(vol) => (0, vol)
        };
        add("price", (good, market.price(good), market.old_price(good), un, vol));
    }
}