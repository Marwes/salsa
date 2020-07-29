//! Test that "on-demand" input pattern works.
//!
//! On-demand inputs are inputs computed lazily on the fly. They are simulated
//! via a b query with zero inputs, which uses `add_synthetic_read` to
//! tweak durability and `invalidate` to clear the input.

use std::{collections::HashMap, sync::Arc};

use crossbeam::atomic::AtomicCell;

use salsa::{Database as _, Durability};

use gluon_salsa as salsa; #[salsa::query_group(QueryGroupStorage)]
trait QueryGroup: salsa::Database + AsRef<HashMap<u32, u32>> {
    fn a(&self, x: u32) -> u32;
    fn b(&self, x: u32) -> u32;
    fn c(&self, x: u32) -> u32;
}

fn a(db: &dyn QueryGroup, x: u32) -> u32 {
    let durability = if x % 2 == 0 {
        Durability::LOW
    } else {
        Durability::HIGH
    };
    db.salsa_runtime_mut().report_synthetic_read(durability);
    let external_state: &HashMap<u32, u32> = db.as_ref();
    external_state[&x]
}

fn b(db: &dyn QueryGroup, x: u32) -> u32 {
    db.a(x)
}

fn c(db: &dyn QueryGroup, x: u32) -> u32 {
    db.b(x)
}

#[salsa::database(QueryGroupStorage)]
#[derive(Default)]
struct Database {
    storage: salsa::Storage<Self>,
    external_state: HashMap<u32, u32>,
    on_event: Option<Box<dyn Fn(salsa::Event)>>,
}

impl salsa::Database for Database {
    fn salsa_event(&self, event: salsa::Event) {
        if let Some(cb) = &self.on_event {
            cb(event)
        }
    }
}

impl AsRef<HashMap<u32, u32>> for Database {
    fn as_ref(&self) -> &HashMap<u32, u32> {
        &self.external_state
    }
}

#[test]
fn on_demand_input_works() {
    let mut db = Database::default();

    db.external_state.insert(1, 10);
    assert_eq!(db.b(1), 10);
    assert_eq!(db.a(1), 10);

    // We changed external state, but haven't signaled about this yet,
    // so we expect to see the old answer
    db.external_state.insert(1, 92);
    assert_eq!(db.b(1), 10);
    assert_eq!(db.a(1), 10);

    AQuery.in_db_mut(&mut db).invalidate(&1);
    assert_eq!(db.b(1), 92);
    assert_eq!(db.a(1), 92);
}

#[test]
fn on_demand_input_durability() {
    let mut db = Database::default();
    db.external_state.insert(1, 10);
    db.external_state.insert(2, 20);
    assert_eq!(db.b(1), 10);
    assert_eq!(db.b(2), 20);

    let validated = Arc::new(AtomicCell::new(0i32));
    db.on_event = Some(Box::new({
        let validated = Arc::clone(&validated);
        move |event| match event.kind {
            salsa::EventKind::DidValidateMemoizedValue { .. } => {
                validated.fetch_add(1);
            }
            _ => (),
        }
    }));

    db.salsa_runtime_mut().synthetic_write(Durability::LOW);
    validated.store(0);
    assert_eq!(db.c(1), 10);
    assert_eq!(db.c(2), 20);
    assert_eq!(validated.load(), 2);

    db.salsa_runtime_mut().synthetic_write(Durability::HIGH);
    validated.store(0);
    assert_eq!(db.c(1), 10);
    assert_eq!(db.c(2), 20);
    assert_eq!(validated.load(), 4);
}
