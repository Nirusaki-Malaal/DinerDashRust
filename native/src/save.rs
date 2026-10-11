use diner_dash_engine::{
    game::GameManager,
    save::{SaveData, SaveManager},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(target_os = "android")]
static SAVE_DIRECTORY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_quad_1native_QuadNative_setSaveDirectory(
    env: *mut jni_sys::JNIEnv,
    _: jni_sys::jclass,
    path: jni_sys::jstring,
) {
    let ptr = (**env).GetStringUTFChars.unwrap()(env, path, std::ptr::null_mut());
    if !ptr.is_null() {
        let text = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
        let _ = SAVE_DIRECTORY.set(PathBuf::from(text));
        (**env).ReleaseStringUTFChars.unwrap()(env, path, ptr);
    }
}

fn directory() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        SAVE_DIRECTORY
            .get()
            .expect("Android files directory initialized before game")
            .clone()
    }
    #[cfg(not(target_os = "android"))]
    {
        dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Nirusaki")
            .join("DinerDash")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileStore {
    pub active_profile: String,
    pub profiles: Vec<String>,
}

pub fn get_profile_store() -> ProfileStore {
    let dir = directory();
    let path = dir.join("profiles.json");
    if path.exists() {
        if let Ok(data) = std::fs::read(&path) {
            if let Ok(store) = serde_json::from_slice::<ProfileStore>(&data) {
                if !store.profiles.is_empty() {
                    return store;
                }
            }
        }
    }
    let default_store = ProfileStore {
        active_profile: "Player 1".to_string(),
        profiles: vec!["Player 1".to_string()],
    };
    save_profile_store(&default_store);
    default_store
}

pub fn save_profile_store(store: &ProfileStore) {
    let dir = directory();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(bytes) = serde_json::to_vec_pretty(store) {
        let _ = std::fs::write(dir.join("profiles.json"), bytes);
    }
}

pub fn list_profiles() -> Vec<String> {
    get_profile_store().profiles
}

pub fn get_active_profile() -> String {
    get_profile_store().active_profile
}

pub fn set_active_profile(name: &str) {
    let mut store = get_profile_store();
    if store.profiles.iter().any(|p| p == name) {
        store.active_profile = name.to_string();
        save_profile_store(&store);
    }
}

pub fn create_profile(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let mut store = get_profile_store();
    if store.profiles.len() >= 10 || store.profiles.iter().any(|p| p == trimmed) {
        return false;
    }
    store.profiles.push(trimmed.to_string());
    store.active_profile = trimmed.to_string();
    save_profile_store(&store);
    true
}

pub fn delete_profile(name: &str) -> bool {
    let mut store = get_profile_store();
    if store.profiles.len() <= 1 {
        return false;
    }
    if let Some(pos) = store.profiles.iter().position(|p| p == name) {
        store.profiles.remove(pos);
        let path = profile_save_path(name);
        let _ = std::fs::remove_file(path);
        if store.active_profile == name {
            store.active_profile = store.profiles[0].clone();
        }
        save_profile_store(&store);
        true
    } else {
        false
    }
}

fn profile_save_path(name: &str) -> PathBuf {
    let safe_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect();
    let trimmed = safe_name.trim();
    let filename = if trimmed.is_empty() {
        "save_default.json".to_string()
    } else {
        format!("save_{trimmed}.json")
    };
    directory().join(filename)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HighScoreEntry {
    pub name: String,
    pub score: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HighScores {
    pub career: Vec<HighScoreEntry>,
    pub endless_easy: Vec<HighScoreEntry>,
    pub endless_medium: Vec<HighScoreEntry>,
    pub endless_hard: Vec<HighScoreEntry>,
}

impl Default for HighScores {
    fn default() -> Self {
        Self {
            career: vec![
                HighScoreEntry { name: "Margaret".into(), score: 20000 },
                HighScoreEntry { name: "Indra".into(), score: 15000 },
                HighScoreEntry { name: "Jorge".into(), score: 10000 },
                HighScoreEntry { name: "Tamesha".into(), score: 7500 },
                HighScoreEntry { name: "Aziz".into(), score: 6000 },
                HighScoreEntry { name: "Jack".into(), score: 5000 },
                HighScoreEntry { name: "Aiden".into(), score: 4000 },
                HighScoreEntry { name: "Milagros".into(), score: 3000 },
                HighScoreEntry { name: "Wu".into(), score: 2000 },
                HighScoreEntry { name: "Suzie".into(), score: 1000 },
            ],
            endless_easy: vec![
                HighScoreEntry { name: "Margaret".into(), score: 2000 },
                HighScoreEntry { name: "Indra".into(), score: 1500 },
                HighScoreEntry { name: "Jorge".into(), score: 1000 },
                HighScoreEntry { name: "Tamesha".into(), score: 750 },
                HighScoreEntry { name: "Aziz".into(), score: 500 },
                HighScoreEntry { name: "Jack".into(), score: 400 },
                HighScoreEntry { name: "Aiden".into(), score: 300 },
                HighScoreEntry { name: "Milagros".into(), score: 200 },
                HighScoreEntry { name: "Wu".into(), score: 100 },
                HighScoreEntry { name: "Suzie".into(), score: 50 },
            ],
            endless_medium: vec![
                HighScoreEntry { name: "Margaret".into(), score: 4000 },
                HighScoreEntry { name: "Indra".into(), score: 3000 },
                HighScoreEntry { name: "Jorge".into(), score: 2000 },
                HighScoreEntry { name: "Tamesha".into(), score: 1500 },
                HighScoreEntry { name: "Aziz".into(), score: 1000 },
                HighScoreEntry { name: "Jack".into(), score: 800 },
                HighScoreEntry { name: "Aiden".into(), score: 600 },
                HighScoreEntry { name: "Milagros".into(), score: 400 },
                HighScoreEntry { name: "Wu".into(), score: 200 },
                HighScoreEntry { name: "Suzie".into(), score: 100 },
            ],
            endless_hard: vec![
                HighScoreEntry { name: "Margaret".into(), score: 6000 },
                HighScoreEntry { name: "Indra".into(), score: 4500 },
                HighScoreEntry { name: "Jorge".into(), score: 3000 },
                HighScoreEntry { name: "Tamesha".into(), score: 2250 },
                HighScoreEntry { name: "Aziz".into(), score: 1500 },
                HighScoreEntry { name: "Jack".into(), score: 1200 },
                HighScoreEntry { name: "Aiden".into(), score: 900 },
                HighScoreEntry { name: "Milagros".into(), score: 600 },
                HighScoreEntry { name: "Wu".into(), score: 300 },
                HighScoreEntry { name: "Suzie".into(), score: 150 },
            ],
        }
    }
}

pub fn get_high_scores() -> HighScores {
    let dir = directory();
    let path = dir.join("high_scores.json");
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(scores) = serde_json::from_slice::<HighScores>(&bytes) {
                return scores;
            }
        }
    }
    let default_scores = HighScores::default();
    save_high_scores(&default_scores);
    default_scores
}

pub fn save_high_scores(scores: &HighScores) {
    let dir = directory();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(bytes) = serde_json::to_vec_pretty(scores) {
        let _ = std::fs::write(dir.join("high_scores.json"), bytes);
    }
}

pub fn record_high_score(mode: &str, name: &str, score: u32) -> Option<usize> {
    let mut scores = get_high_scores();
    let list = match mode {
        "career" => &mut scores.career,
        "endless_easy" => &mut scores.endless_easy,
        "endless_medium" => &mut scores.endless_medium,
        "endless_hard" => &mut scores.endless_hard,
        _ => return None,
    };
    let pos = list.iter().position(|e| score > e.score).unwrap_or(list.len());
    if pos < 10 {
        list.insert(pos, HighScoreEntry { name: name.to_string(), score });
        list.truncate(10);
        save_high_scores(&scores);
        Some(pos + 1)
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameOptions {
    pub music: bool,
    pub sfx: bool,
    pub fullscreen: bool,
    pub stretch_display: bool,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            music: true,
            sfx: true,
            fullscreen: false,
            stretch_display: cfg!(target_os = "android"),
        }
    }
}

pub fn get_options() -> GameOptions {
    let dir = directory();
    let path = dir.join("options.json");
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(opts) = serde_json::from_slice::<GameOptions>(&bytes) {
                return opts;
            }
        }
    }
    let default_opts = GameOptions::default();
    save_options(&default_opts);
    default_opts
}

pub fn save_options(opts: &GameOptions) {
    let dir = directory();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(bytes) = serde_json::to_vec_pretty(opts) {
        let _ = std::fs::write(dir.join("options.json"), bytes);
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    profile: SaveData,
    session: GameManager,
}

pub fn load(profile: &mut SaveManager) -> Option<GameManager> {
    let active = get_active_profile();
    let mut path = profile_save_path(&active);
    if !path.exists() {
        // Fallback to legacy save.json if it exists
        let legacy = directory().join("save.json");
        if legacy.exists() {
            path = legacy;
        } else {
            return None;
        }
    }
    let result = (|| -> Result<GameManager, Box<dyn std::error::Error>> {
        let snap: Snapshot = serde_json::from_slice(&std::fs::read(&path)?)?;
        if snap.version != 1 {
            return Err("Unsupported save version".into());
        }
        profile.from_json(&serde_json::to_string(&snap.profile)?)?;
        Ok(snap.session)
    })();
    match result {
        Ok(game) => Some(game),
        Err(e) => {
            eprintln!("Cannot load save: {e}");
            None
        }
    }
}

pub fn store(profile: &SaveManager, game: &GameManager) -> Result<(), Box<dyn std::error::Error>> {
    let dir = directory();
    std::fs::create_dir_all(&dir)?;
    let active = get_active_profile();
    let path = profile_save_path(&active);
    let bytes = serde_json::to_vec(&Snapshot {
        version: 1,
        profile: profile.data().clone(),
        session: game.clone(),
    })?;
    let temp = dir.join(format!("{}.tmp", path.file_name().unwrap().to_string_lossy()));
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&temp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(temp, path)?;
    Ok(())
}

#[cfg(target_os = "android")]
pub static PAUSE_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_quad_1native_QuadNative_requestPause(
    _: *mut jni_sys::JNIEnv,
    _: jni_sys::jclass,
) {
    PAUSE_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn export(profile: &SaveManager) -> Result<String, Box<dyn std::error::Error>> {
    let dir = directory();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("backup.json");
    std::fs::write(&path, profile.to_json()?)?;
    Ok(format!("Backup: {}", path.display()))
}
pub fn import(profile: &mut SaveManager) -> Result<(), Box<dyn std::error::Error>> {
    profile.from_json(&std::fs::read_to_string(directory().join("backup.json"))?)?;
    Ok(())
}
pub fn reset() -> Result<(), std::io::Error> {
    let active = get_active_profile();
    let path = profile_save_path(&active);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let legacy = directory().join("save.json");
    if legacy.exists() {
        std::fs::remove_file(legacy)?;
    }
    Ok(())
}
