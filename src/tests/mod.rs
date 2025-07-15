pub mod formatting;

mod interpolation;
mod maneuver;
mod parser_3c;
mod parser_3d;
mod test_pool;

#[cfg(feature = "qc")]
mod merge;

#[cfg(feature = "processing")]
mod substract;

//#[cfg(feature = "qc")]
//mod decimation;

//#[cfg(feature = "qc")]
//mod masking;

use log::LevelFilter;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .init();
    });
}
