use hadrone_core::LayoutItem;
use serde::{Deserialize, Serialize};
use std::io::Result;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutSnapshot {
    pub version: u32,
    pub items: Vec<LayoutItem>,
    pub cols: i32,
}

pub trait LayoutStorage: Send + Sync {
    fn save(&self, key: &str, layout: &LayoutSnapshot) -> Result<()>;
    fn load(&self, key: &str) -> Result<Option<LayoutSnapshot>>;
}

// --- Native File Storage ---

#[cfg(not(target_arch = "wasm32"))]
pub struct FileStorage {
    base_path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let path = base_path.into();
        std::fs::create_dir_all(&path).ok();
        Self { base_path: path }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl LayoutStorage for FileStorage {
    fn save(&self, key: &str, layout: &LayoutSnapshot) -> Result<()> {
        let path = self.base_path.join(format!("{}.json", key));
        let data = serde_json::to_string_pretty(layout)?;
        std::fs::write(path, data)
    }

    fn load(&self, key: &str) -> Result<Option<LayoutSnapshot>> {
        let path = self.base_path.join(format!("{}.json", key));
        if !path.exists() {
            return Ok(None);
        }
        let data = std::fs::read_to_string(path)?;
        let layout = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(layout))
    }
}

// --- Browser Local Storage ---

pub struct BrowserStorage {
    _prefix: String,
    #[cfg(target_arch = "wasm32")]
    fallback: std::sync::Mutex<std::collections::HashMap<String, LayoutSnapshot>>,
}

impl BrowserStorage {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            _prefix: prefix.into(),
            #[cfg(target_arch = "wasm32")]
            fallback: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn storage(&self) -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }
}

impl LayoutStorage for BrowserStorage {
    fn save(&self, _key: &str, _layout: &LayoutSnapshot) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage_key = format!("{}:{}", self._prefix, _key);
            let data = serde_json::to_string(_layout)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            if let Some(storage) = self.storage() {
                if storage.set_item(&storage_key, &data).is_ok() {
                    return Ok(());
                }
            }

            // Fallback
            self.fallback
                .lock()
                .unwrap()
                .insert(_key.to_string(), _layout.clone());
        }
        Ok(())
    }

    fn load(&self, _key: &str) -> Result<Option<LayoutSnapshot>> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage_key = format!("{}:{}", self._prefix, _key);

            if let Some(storage) = self.storage() {
                if let Ok(Some(data)) = storage.get_item(&storage_key) {
                    return serde_json::from_str(&data)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e));
                }
            }

            // Fallback
            return Ok(self.fallback.lock().unwrap().get(_key).cloned());
        }

        #[cfg(not(target_arch = "wasm32"))]
        Ok(None)
    }
}

// --- Responsive Breakpoints ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BreakpointConfig {
    pub name: String,
    pub cols: i32,
    pub min_width: i32,
    pub margin: (i32, i32),
    pub row_height: f32,
}

impl Default for BreakpointConfig {
    fn default() -> Self {
        Self {
            name: "lg".into(),
            cols: 12,
            min_width: 1200,
            margin: (10, 10),
            row_height: 100.0,
        }
    }
}

// --- Debounced Auto-Save ---

pub async fn debounce_save(
    storage: &dyn LayoutStorage,
    key: &str,
    layout: Vec<LayoutItem>,
    cols: i32,
    ms: u64,
) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(Duration::from_millis(ms)).await;

    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms as u32).await;

    let _ = storage.save(
        key,
        &LayoutSnapshot {
            version: 1,
            items: layout,
            cols,
        },
    );
}

// --- Responsive Detection ---

#[cfg(feature = "dioxus")]
pub fn use_responsive_grid(
    breakpoints: Vec<BreakpointConfig>,
) -> dioxus::prelude::Signal<BreakpointConfig> {
    use dioxus::prelude::*;

    let initial_bp = {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(width) = window.inner_width() {
                    if let Some(w) = width.as_f64() {
                        let w = w as i32;
                        let mut best_match = &breakpoints[0];
                        for bp in &breakpoints {
                            if w >= bp.min_width && bp.min_width >= best_match.min_width {
                                best_match = bp;
                            }
                        }
                        best_match.clone()
                    } else {
                        breakpoints[0].clone()
                    }
                } else {
                    breakpoints[0].clone()
                }
            } else {
                breakpoints[0].clone()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            breakpoints.last().unwrap_or(&breakpoints[0]).clone()
        }
    };

    let current = use_signal(|| initial_bp);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::{ResizeObserver, ResizeObserverEntry};

        use_effect(move || {
            let breakpoints = breakpoints.clone();
            let mut current = current;

            let closure = Closure::wrap(Box::new(
                move |entries: js_sys::Array, _observer: ResizeObserver| {
                    if let Some(entry) = entries.get(0).dyn_into::<ResizeObserverEntry>().ok() {
                        let width = entry.content_rect().width() as i32;

                        // Find matching breakpoint
                        let mut best_match = &breakpoints[0];
                        for bp in &breakpoints {
                            if width >= bp.min_width && bp.min_width >= best_match.min_width {
                                best_match = bp;
                            }
                        }
                        current.set(best_match.clone());
                    }
                },
            )
                as Box<dyn FnMut(js_sys::Array, ResizeObserver)>);

            let observer = ResizeObserver::new(closure.as_ref().unchecked_ref()).unwrap();

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        observer.observe(&body);
                    }
                }
            }

            closure.forget();
        });
    }

    current
}
