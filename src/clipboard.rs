use parking_lot::Mutex;
use raw_window_handle::RawDisplayHandle;

#[cfg(not(target_arch = "wasm32"))]
use copypasta::{ClipboardContext, ClipboardProvider};

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
mod wasm_clipboard {
    use std::cell::RefCell;
    thread_local! {
        static CLIPBOARD: RefCell<String> =
            RefCell::new(String::new());
    }
    pub fn get() -> String {
        CLIPBOARD.with(|c| c.borrow().clone())
    }
    pub fn set(s: &str) {
        CLIPBOARD
            .with(|c| *c.borrow_mut() = s.to_string());
    }
}

pub struct Clipboard {
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Box<dyn ClipboardProvider>,
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    selection: Option<Box<dyn ClipboardProvider>>,
}

#[derive(Clone, Debug)]
pub enum ClipboardError {
    NotAvailable,
    ProviderError(String),
}

impl Clipboard {
    pub fn get_contents(
    ) -> Result<String, ClipboardError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            CLIPBOARD
                .lock()
                .as_mut()
                .ok_or(ClipboardError::NotAvailable)?
                .clipboard
                .get_contents()
                .map_err(|e| {
                    ClipboardError::ProviderError(
                        e.to_string(),
                    )
                })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(wasm_clipboard::get())
        }
    }

    pub fn set_contents(
        s: String,
    ) -> Result<(), ClipboardError> {
        if s.is_empty() {
            return Err(ClipboardError::ProviderError(
                "content is empty".to_string(),
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            CLIPBOARD
                .lock()
                .as_mut()
                .ok_or(ClipboardError::NotAvailable)?
                .clipboard
                .set_contents(s)
                .map_err(|e| {
                    ClipboardError::ProviderError(
                        e.to_string(),
                    )
                })
        }
        #[cfg(target_arch = "wasm32")]
        {
            wasm_clipboard::set(&s);
            Ok(())
        }
    }

    #[cfg(windows)]
    pub fn get_file_list(
    ) -> Result<Vec<std::path::PathBuf>, ClipboardError> {
        clipboard_win::Clipboard::new_attempts(10)
            .and_then(|x| x.get_file_list())
            .map_err(|e| {
                ClipboardError::ProviderError(
                    e.to_string(),
                )
            })
    }

    pub(crate) unsafe fn init(
        #[allow(unused_variables)] display: RawDisplayHandle,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            *CLIPBOARD.lock() =
                Some(Self::new(display));
        }
    }

    /// # Safety
    /// The `display` must be valid as long as the
    /// returned Clipboard exists.
    #[cfg(not(target_arch = "wasm32"))]
    unsafe fn new(
        #[allow(unused_variables)]
        display: RawDisplayHandle,
    ) -> Self {
        #[cfg(not(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "ios",
            target_os = "android",
        )))]
        {
            if let RawDisplayHandle::Wayland(display) =
                display
            {
                use copypasta::wayland_clipboard;
                let (selection, clipboard) =
                    wayland_clipboard::create_clipboards_from_external(display.display.as_ptr());
                return Self {
                    clipboard: Box::new(clipboard),
                    selection: Some(Box::new(selection)),
                };
            }

            use copypasta::x11_clipboard::{
                Primary, X11ClipboardContext,
            };
            Self {
                clipboard: Box::new(
                    ClipboardContext::new().unwrap(),
                ),
                selection: Some(Box::new(
                    X11ClipboardContext::<Primary>::new()
                        .unwrap(),
                )),
            }
        }

        #[cfg(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "ios",
            target_os = "android",
        ))]
        return Self {
            clipboard: Box::new(
                ClipboardContext::new().unwrap(),
            ),
            selection: None,
        };
    }
}
