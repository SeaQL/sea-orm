use std::io::Write as _;

/// SeaORM Entity for state persistence
mod state {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "state")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub digits: u32,
        pub boxes: Digits,
        pub i: u32,
        pub nines: u32,
        pub predigit: u8,
        pub have_predigit: bool,
        pub count: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct Digits(pub Vec<u32>);

    impl ActiveModelBehavior for ActiveModel {}
}

mod run_log {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "run_log")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub digits: u32,
        pub pi_digits: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct Digits(pub Vec<u32>);

    impl ActiveModelBehavior for ActiveModel {}
}

fn pi_spigot(digits: u32) {
    use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, NotSet, Set, TransactionTrait};

    // open database file, create if not exists
    let db = &sea_orm::Database::connect("sqlite://pi.sqlite").unwrap();
    db.get_schema_builder()
        .register(state::Entity)
        .register(run_log::Entity)
        .sync(db) // create table if not exists
        .unwrap();

    if digits == 0 {
        println!("3");
        return;
    }

    let len = digits as usize * 10 / 3 + 1;

    let state = match state::Entity::find_by_id(digits).one(db).unwrap() {
        Some(state) => {
            println!("resuming from {}th digit", state.count);
            state
        }
        None => state::Model {
            digits,
            boxes: state::Digits(vec![2u32; len]),
            i: 0,
            nines: 0,
            predigit: 0,
            have_predigit: false,
            count: 0,
        }
        .into_active_model()
        .insert(db)
        .unwrap(),
    };
    let mut run_log = run_log::ActiveModel {
        id: NotSet,
        digits: Set(digits),
        pi_digits: Set("".to_owned()),
    }
    .save(db)
    .unwrap();

    let mut boxes = state.boxes.0;
    let mut nines = state.nines;
    let mut predigit = state.predigit;
    let mut have_predigit = state.have_predigit;
    let mut count = state.count;
    let mut s = String::new();

    for i in (0..(digits + 1)).skip(state.i as usize) {
        if count % 100 == 0 {
            std::io::stdout().flush().unwrap();
            // save checkpoint
            let txn = db.begin().unwrap();
            state::Model {
                digits,
                boxes: state::Digits(boxes.clone()),
                i,
                nines: nines,
                predigit: predigit,
                have_predigit: have_predigit,
                count: count,
            }
            .into_active_model()
            .reset_all() // we want to update all fields
            .update(&txn)
            .unwrap();

            run_log.pi_digits = Set(s.to_owned());
            run_log = run_log.save(&txn).unwrap();

            txn.commit().unwrap();
        }

        let mut carry: u32 = 0;
        // work backwards over boxes 1..len-1
        for j in (1..len).rev() {
            let j_u = j as u32;
            let x = boxes[j] * 10 + carry;
            let q = x / (2 * j_u + 1);
            boxes[j] = x % (2 * j_u + 1);
            carry = q * j_u;
        }
        let x = boxes[0] * 10 + carry;
        let q = (x / 10) as u8; // 0..10
        boxes[0] = x % 10;

        if q == 9 {
            nines += 1;
        } else if q == 10 {
            // increment previous printed digit
            if have_predigit {
                new_digit(&mut s, predigit + 1);
                count += 1;
            } else {
                // first digit becomes 1
                new_digit(&mut s, 1);
                count += 1;
                have_predigit = true;
            }
            for _ in 0..nines {
                new_digit(&mut s, 0);
                count += 1;
            }
            predigit = 0;
            nines = 0;
        } else {
            if have_predigit {
                new_digit(&mut s, predigit);
                count += 1;
                if count == 1 {
                    println!(".");
                }
            } else {
                have_predigit = true;
            }
            predigit = q;
            for _ in 0..nines {
                new_digit(&mut s, 9);
                count += 1;
            }
            nines = 0;
        }
    }

    // append last predigit
    new_digit(&mut s, predigit);

    println!();
    run_log.pi_digits = Set(s.to_owned());
    run_log.save(db).unwrap();
}

fn new_digit(s: &mut String, digit: u8) {
    let c = (b'0' + digit) as char;
    s.push(c);
    print!("{c}");
}

fn main() {
    pi_spigot(1_000_000);
}
