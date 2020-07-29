use crate::queries;
use crossbeam::atomic::AtomicCell;

use gluon_salsa as salsa;

#[salsa::database(queries::GroupStruct)]
#[derive(Default)]
pub(crate) struct DatabaseImpl {
    storage: salsa::Storage<Self>,
    counter: Cell<usize>,
}

impl queries::Counter for DatabaseImpl {
    fn increment(&self) -> usize {
        self.counter.fetch_add(1)
    }
}

impl salsa::Database for DatabaseImpl {}
