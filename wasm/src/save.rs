//! Web Storage & IndexedDB Persistence Layer for Diner Dash WASM
//!
//! Provides dual-backed persistence:
//! 1. Synchronous `localStorage` for instant startup hydration and mid-shift snapshots.
//! 2. Asynchronous `IndexedDB` (`DinerDashDB` -> `saves` object store) for durable storage.

use diner_dash_engine::game::GameManager;
use diner_dash_engine::save::{SaveData, SaveManager};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Event, IdbDatabase, IdbOpenDbRequest, IdbTransactionMode, Storage};

const SAVE_STORAGE_KEY: &str = "diner_dash_save";
const SESSION_STORAGE_KEY: &str = "diner_dash_active_session_v3";
const IDB_DB_NAME: &str = "DinerDashDB";
const IDB_STORE_NAME: &str = "saves";

pub struct WebPersistence;

impl WebPersistence {
    fn local_storage() -> Result<Storage, JsValue> {
        let win = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        win.local_storage()?
            .ok_or_else(|| JsValue::from_str("localStorage unavailable"))
    }

    /// Save career/endless profile data to both localStorage and IndexedDB
    pub fn save_profile(save_mgr: &mut SaveManager) -> Result<(), JsValue> {
        save_mgr
            .save()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        if let Ok(json) = save_mgr.to_json() {
            Self::write_idb_async("profile_slot_1", &json);
        }
        Ok(())
    }

    /// Load career/endless profile data from localStorage
    pub fn load_profile(save_mgr: &mut SaveManager) -> Result<bool, JsValue> {
        let storage = Self::local_storage()?;
        if let Some(raw) = storage.get_item(SAVE_STORAGE_KEY)? {
            save_mgr
                .from_json(&raw)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Save the active mid-shift `GameManager` state so page refreshes resume seamlessly
    pub fn save_session_snapshot(game: &GameManager) -> Result<(), JsValue> {
        let storage = Self::local_storage()?;
        let payload =
            serde_json::to_string(game).map_err(|e| JsValue::from_str(&e.to_string()))?;
        storage.set_item(SESSION_STORAGE_KEY, &payload)?;
        Self::write_idb_async("session_snapshot_v2", &payload);
        Ok(())
    }

    /// Load an active mid-shift `GameManager` snapshot if one exists
    pub fn load_session_snapshot() -> Result<Option<GameManager>, JsValue> {
        let storage = Self::local_storage()?;
        if let Some(raw) = storage.get_item(SESSION_STORAGE_KEY)? {
            if let Ok(game) = serde_json::from_str::<GameManager>(&raw) {
                return Ok(Some(game));
            }
        }
        Ok(None)
    }

    /// Clear saved mid-shift snapshot (e.g. when starting a new shift)
    pub fn clear_session_snapshot() -> Result<(), JsValue> {
        let storage = Self::local_storage()?;
        storage.remove_item(SESSION_STORAGE_KEY)?;
        Self::delete_idb_async("session_snapshot_v2");
        Ok(())
    }

    /// Export full profile save data as JSON string
    pub fn export_save_json(save_mgr: &SaveManager) -> Result<String, JsValue> {
        save_mgr
            .to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Import full profile save data from JSON string and persist immediately
    pub fn import_save_json(save_mgr: &mut SaveManager, json: &str) -> Result<(), JsValue> {
        let parsed: SaveData =
            serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let canonical =
            serde_json::to_string(&parsed).map_err(|e| JsValue::from_str(&e.to_string()))?;
        save_mgr
            .from_json(&canonical)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Self::save_profile(save_mgr)
    }

    /// Delete a key from IndexedDB (`DinerDashDB` / `saves`)
    pub fn delete_idb_async(key: &str) {
        let Some(win) = web_sys::window() else {
            return;
        };
        let Ok(Some(idb)) = win.indexed_db() else {
            return;
        };
        let Ok(open_req) = idb.open_with_u32(IDB_DB_NAME, 1) else {
            return;
        };

        let on_upgrade = Closure::once(move |event: Event| {
            if let Some(target) = event.target() {
                if let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(res) = req.result() {
                        if let Ok(db) = res.dyn_into::<IdbDatabase>() {
                            let _ = db.create_object_store(IDB_STORE_NAME);
                        }
                    }
                }
            }
        });
        open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let key_owned = key.to_string();
        let on_success = Closure::once(move |event: Event| {
            if let Some(target) = event.target() {
                if let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(res) = req.result() {
                        if let Ok(db) = res.dyn_into::<IdbDatabase>() {
                            if let Ok(tx) = db.transaction_with_str_and_mode(
                                IDB_STORE_NAME,
                                IdbTransactionMode::Readwrite,
                            ) {
                                if let Ok(store) = tx.object_store(IDB_STORE_NAME) {
                                    let _ = store.delete(&JsValue::from_str(&key_owned));
                                }
                            }
                        }
                    }
                }
            }
        });
        open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();
    }

    /// Mirror a key-value JSON blob into IndexedDB (`DinerDashDB` / `saves`)
    pub fn write_idb_async(key: &str, json_payload: &str) {
        let Some(win) = web_sys::window() else {
            return;
        };
        let Ok(Some(idb)) = win.indexed_db() else {
            return;
        };
        let Ok(open_req) = idb.open_with_u32(IDB_DB_NAME, 1) else {
            return;
        };

        let on_upgrade = Closure::once(move |event: Event| {
            if let Some(target) = event.target() {
                if let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(res) = req.result() {
                        if let Ok(db) = res.dyn_into::<IdbDatabase>() {
                            let _ = db.create_object_store(IDB_STORE_NAME);
                        }
                    }
                }
            }
        });
        open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let key_owned = key.to_string();
        let val_owned = json_payload.to_string();
        let on_success = Closure::once(move |event: Event| {
            if let Some(target) = event.target() {
                if let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(res) = req.result() {
                        if let Ok(db) = res.dyn_into::<IdbDatabase>() {
                            if let Ok(tx) = db.transaction_with_str_and_mode(
                                IDB_STORE_NAME,
                                IdbTransactionMode::Readwrite,
                            ) {
                                if let Ok(store) = tx.object_store(IDB_STORE_NAME) {
                                    let _ = store.put_with_key(
                                        &JsValue::from_str(&val_owned),
                                        &JsValue::from_str(&key_owned),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
        open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();
    }

    /// Hydrate `localStorage` from IndexedDB on startup if `localStorage` was cleared
    pub fn sync_from_idb_to_local_storage() {
        let Some(win) = web_sys::window() else {
            return;
        };
        let Ok(Some(idb)) = win.indexed_db() else {
            return;
        };
        let Ok(open_req) = idb.open_with_u32(IDB_DB_NAME, 1) else {
            return;
        };

        let on_upgrade = Closure::once(move |event: Event| {
            if let Some(target) = event.target() {
                if let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(res) = req.result() {
                        if let Ok(db) = res.dyn_into::<IdbDatabase>() {
                            let _ = db.create_object_store(IDB_STORE_NAME);
                        }
                    }
                }
            }
        });
        open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let on_success = Closure::once(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() else {
                return;
            };
            let Ok(res) = req.result() else {
                return;
            };
            let Ok(db) = res.dyn_into::<IdbDatabase>() else {
                return;
            };
            let Ok(tx) =
                db.transaction_with_str_and_mode(IDB_STORE_NAME, IdbTransactionMode::Readonly)
            else {
                return;
            };
            let Ok(store) = tx.object_store(IDB_STORE_NAME) else {
                return;
            };
            for (idb_key, ls_key) in [
                ("profile_slot_1", SAVE_STORAGE_KEY),
                ("session_snapshot_v2", SESSION_STORAGE_KEY),
            ] {
                if let Ok(get_req) = store.get(&JsValue::from_str(idb_key)) {
                    let ls_key_owned = ls_key.to_string();
                    let on_get = Closure::once(move |ev: Event| {
                        if let Some(t) = ev.target() {
                            if let Ok(r) = t.dyn_into::<web_sys::IdbRequest>() {
                                if let Ok(val) = r.result() {
                                    if let Some(json_str) = val.as_string() {
                                        if let Ok(storage) = Self::local_storage() {
                                            if storage.get_item(&ls_key_owned).ok().flatten().is_none()
                                            {
                                                let _ = storage.set_item(&ls_key_owned, &json_str);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                    get_req.set_onsuccess(Some(on_get.as_ref().unchecked_ref()));
                    on_get.forget();
                }
            }
        });
        open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();
    }
}
