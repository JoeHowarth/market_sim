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

fn main() {
    init_recorder("buy_sell", true);
    let tasks = vec![
        Task::new("Bake", &[(Grain, 40)], (Food, 10)),
        Task::new("Farm", &[], (Grain, 10)),
    ];

    let agents = Agent::pre_made(10);

    let market = ClearingMarket::new(hashmap! {
        Food => 15,
        Grain => 5,
    });

    register("deaths", &["agent_id"]);
    register("tasks", &["task_name", "task_value", "revenue", "cost", "agent_id"]);
    register("price", &["good", "price", "unexecuted", "volume"]);
    register("agent_info", &["agent_id", "cash", "food", "grain"]);
    register("utility", &["agent_id", "utility", "food_consumed"]);

    run(tasks, agents, market, 20);

    flush();
}

fn run(tasks: Vec<Task>,
       mut agents: HashMap<AgentId, Agent>,
       mut market: impl Market,
       max_iters: u16) {
    let mut dead = HashSet::with_capacity(100);
    let rng = SmallRng::from_entropy();

    let food_mu = MU(vec![60_i16, 30, 25, 20, 15, 10, 5, 2, 1]);
    let food_utils = food_mu.utility(0);

    for i in 0..max_iters {
        set_tick(i);
        println!("{}", i);
//        add("price", ("Food", market.price(Food) ));
//        add("price", ("Grain", market.price(Grain)));


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

        // consume food
        for a in agents.values_mut() {
            let food = a.res[&Food] as usize;
            if food <= 1 {
                dead.insert(a.id);
            }
            // they've already traded what they want, so eat it all!
            // later, factor in discounted consumption
            add("utility", (a.id, food_utils[food.min(5)], food.min(5)));
            *a.res.get_mut(&Food).unwrap() -= 5.min(food as i16);
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
        add("price", (good, market.price(good), un, vol));
    }
}