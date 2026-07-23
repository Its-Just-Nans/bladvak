//! Run a example app
//!
//! ```
//! cargo run --example example_app
//! ```

use bladvak::app::{Bladvak, BladvakApp, MainResult};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct ExampleApp {}

impl BladvakApp<'_> for ExampleApp {
    fn try_new_with_args(
        saved_state: Self,
        _cc: &eframe::CreationContext<'_>,
        _args: &[String],
        _error_manager: &mut bladvak::ErrorManager,
    ) -> Result<Self, bladvak::AppError> {
        Ok(saved_state)
    }

    fn version() -> String {
        "0.0.1".to_string()
    }

    fn name() -> String {
        "example_app".to_string()
    }
}

fn main() -> MainResult {
    Bladvak::<ExampleApp>::bladvak_main()
}
