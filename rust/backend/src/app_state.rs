use once_cell::sync::OnceCell;

use crate::persist::in_mem::InMemoryPersist;

pub struct AppState {
    pub persistence: InMemoryPersist,
}

static APP_STATE: OnceCell<AppState> = OnceCell::new();

pub async fn get_app_state() -> &'static AppState {
    APP_STATE.get_or_init(|| {
        let persistence = InMemoryPersist::new();
        AppState { persistence }
    })
}
