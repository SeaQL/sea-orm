use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, NotSet, Set,
    TransactionTrait,
};

pub mod state {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "state")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub digits: u32,
        pub boxes: JsonVec,
        pub i: u32,
        pub nines: u32,
        pub predigit: u8,
        pub have_predigit: bool,
        pub count: u32,
        #[sea_orm(column_type = "Text")]
        pub result: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct JsonVec(pub Vec<u32>);

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod run_log {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "run_log")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub digits: u32,
        #[sea_orm(column_type = "Text")]
        pub pi_digits: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Tracks the mutable computation state of the spigot algorithm.
pub struct PiSpigot {
    digits: u32,
    boxes: Vec<u32>,
    nines: u32,
    predigit: u8,
    have_predigit: bool,
    count: u32,
    result: String,
    start_i: u32,
}

impl PiSpigot {
    /// Create a new computation for the given number of decimal digits (after "3.").
    pub fn new(digits: u32) -> Self {
        let len = digits as usize * 10 / 3 + 1;
        Self {
            digits,
            boxes: vec![2u32; len],
            nines: 0,
            predigit: 0,
            have_predigit: false,
            count: 0,
            result: String::new(),
            start_i: 0,
        }
    }

    /// Load from a checkpoint if one exists, otherwise create fresh.
    pub fn resume(db: &DatabaseConnection, digits: u32) -> Result<Self, sea_orm::DbErr> {
        db.get_schema_builder()
            .register(state::Entity)
            .register(run_log::Entity)
            .sync(db)?;

        match state::Entity::find_by_id(digits).one(db)? {
            Some(s) => {
                eprintln!(
                    "Resuming from checkpoint (iteration {}, {} digits result)",
                    s.i, s.count
                );
                Ok(Self::from_state(s))
            }
            None => Ok(Self::new(digits)),
        }
    }

    /// Compute all digits without database persistence. Returns the decimal
    /// digits of pi after "3." (e.g. "14159265..." for 8 digits).
    pub fn compute(mut self) -> String {
        for _ in 0..=self.digits {
            self.step();
        }
        self.finalize();
        self.result
    }

    /// Compute with database persistence, checkpointing every `checkpoint_interval` iterations.
    /// Returns the fractional digits of pi (after "3.").
    pub fn compute_with_db(
        mut self,
        db: &DatabaseConnection,
        checkpoint_interval: u32,
    ) -> Result<String, sea_orm::DbErr> {
        db.get_schema_builder()
            .register(state::Entity)
            .register(run_log::Entity)
            .sync(db)?;

        let mut run_log = run_log::ActiveModel {
            id: NotSet,
            digits: Set(self.digits),
            pi_digits: Set(String::new()),
        }
        .save(db)?;

        for i in self.start_i..=self.digits {
            self.step();

            if checkpoint_interval > 0 && self.count > 0 && self.count % checkpoint_interval == 0 {
                let txn = db.begin()?;
                state::Entity::delete_by_id(self.digits).exec(&txn)?;
                self.to_state(i + 1).into_active_model().insert(&txn)?;
                run_log.pi_digits = Set(self.result.clone());
                run_log = run_log.save(&txn)?;
                txn.commit()?;
                let start = self
                    .result
                    .len()
                    .saturating_sub(checkpoint_interval as usize);
                eprintln!("[{}] {}", self.count, &self.result[start..]);
            }
        }

        self.finalize();

        state::Entity::delete_by_id(self.digits).exec(db)?;

        run_log.pi_digits = Set(self.result.clone());
        run_log.save(db)?;

        Ok(self.result)
    }

    /// Resume from a persisted state, recovering previously result digits.
    fn from_state(s: state::Model) -> Self {
        Self {
            digits: s.digits,
            boxes: s.boxes.0,
            nines: s.nines,
            predigit: s.predigit,
            have_predigit: s.have_predigit,
            count: s.count,
            result: s.result,
            start_i: s.i,
        }
    }

    /// Build a state model for checkpoint persistence.
    fn to_state(&self, i: u32) -> state::Model {
        state::Model {
            digits: self.digits,
            boxes: state::JsonVec(self.boxes.clone()),
            i,
            nines: self.nines,
            predigit: self.predigit,
            have_predigit: self.have_predigit,
            count: self.count,
            result: self.result.clone(),
        }
    }

    fn push_digit(&mut self, digit: u8) {
        self.result.push((b'0' + digit) as char);
        self.count += 1;
    }

    /// Run one iteration of the spigot algorithm, producing zero or more digits.
    fn step(&mut self) {
        let len = self.boxes.len();
        let mut carry: u32 = 0;

        for j in (1..len).rev() {
            let j_u = j as u32;
            let x = self.boxes[j] * 10 + carry;
            self.boxes[j] = x % (2 * j_u + 1);
            carry = (x / (2 * j_u + 1)) * j_u;
        }
        let x = self.boxes[0] * 10 + carry;
        let q = (x / 10) as u8;
        self.boxes[0] = x % 10;

        if q == 9 {
            self.nines += 1;
        } else if q == 10 {
            if self.have_predigit {
                self.push_digit(self.predigit + 1);
            } else {
                self.push_digit(1);
                self.have_predigit = true;
            }
            for _ in 0..self.nines {
                self.push_digit(0);
            }
            self.predigit = 0;
            self.nines = 0;
        } else {
            if self.have_predigit {
                self.push_digit(self.predigit);
            } else {
                self.have_predigit = true;
            }
            self.predigit = q;
            for _ in 0..self.nines {
                self.push_digit(9);
            }
            self.nines = 0;
        }
    }

    /// Flush the final predigit and strip the leading "3" so the result
    /// contains only fractional digits (e.g. "14159..." not "314159...").
    fn finalize(&mut self) {
        self.push_digit(self.predigit);
        for _ in 0..self.nines {
            self.push_digit(9);
        }
        self.nines = 0;
        // The spigot algorithm produces digits starting with 3; strip it.
        if self.result.starts_with('3') {
            self.result.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PI_1000: &str = "\
        14159265358979323846264338327950288419716939937510\
        58209749445923078164062862089986280348253421170679\
        82148086513282306647093844609550582231725359408128\
        48111745028410270193852110555964462294895493038196\
        44288109756659334461284756482337867831652712019091\
        45648566923460348610454326648213393607260249141273\
        72458700660631558817488152092096282925409171536436\
        78925903600113305305488204665213841469519415116094\
        33057270365759591953092186117381932611793105118548\
        07446237996274956735188575272489122793818301194912\
        98336733624406566430860213949463952247371907021798\
        60943702770539217176293176752384674818467669405132\
        00056812714526356082778577134275778960917363717872\
        14684409012249534301465495853710507922796892589235\
        42019956112129021960864034418159813629774771309960\
        51870721134999999837297804995105973173281609631859\
        50244594553469083026425223082533446850352619311881\
        71010003137838752886587533208381420617177669147303\
        59825349042875546873115956286388235378759375195778\
        18577805321712268066130019278766111959092164201989";

    #[test]
    fn test_compute_10_digits() {
        let result = PiSpigot::new(10).compute();
        assert_eq!(&result[..10], &PI_1000[..10]);
    }

    #[test]
    fn test_compute_100_digits() {
        let result = PiSpigot::new(100).compute();
        assert_eq!(&result[..100], &PI_1000[..100]);
    }

    #[test]
    fn test_compute_1000_digits() {
        let result = PiSpigot::new(1000).compute();
        assert_eq!(&result[..1000], PI_1000);
    }

    #[test]
    fn test_compute_with_db() {
        let db = sea_orm::Database::connect("sqlite::memory:").unwrap();
        let spigot = PiSpigot::new(100);
        let result = spigot.compute_with_db(&db, 10).unwrap();
        assert_eq!(&result[..100], &PI_1000[..100]);
    }

    #[test]
    fn test_checkpoint_resume() {
        let db = sea_orm::Database::connect("sqlite::memory:").unwrap();

        db.get_schema_builder()
            .register(state::Entity)
            .register(run_log::Entity)
            .sync(&db)
            .unwrap();

        // Phase 1: step 0..100, checkpoint
        let mut spigot = PiSpigot::new(1000);
        for _ in 0..100 {
            spigot.step();
        }
        spigot
            .to_state(100)
            .into_active_model()
            .insert(&db)
            .unwrap();

        // Phase 2: resume, step 100..500, checkpoint again
        let mut spigot = PiSpigot::resume(&db, 1000).unwrap();
        assert_eq!(spigot.start_i, 100);
        for _ in 100..500 {
            spigot.step();
        }
        state::Entity::delete_by_id(1000u32).exec(&db).unwrap();
        spigot
            .to_state(500)
            .into_active_model()
            .insert(&db)
            .unwrap();

        // Phase 3: resume from second checkpoint, finish
        let resumed = PiSpigot::resume(&db, 1000).unwrap();
        assert_eq!(resumed.start_i, 500);
        let result = resumed.compute_with_db(&db, 0).unwrap();
        assert_eq!(result, PI_1000);
    }
}
