use std::{
    fmt::Debug,
    hash::{self, DefaultHasher, Hash, Hasher},
    sync::Mutex,
};

use crate::sync::Rc;

/// The trait that any clipboard provider must implement
pub trait ClipboardProvider {
    /// Set the contents of the clipboard to the given string slice
    fn set_str(&mut self, v: &str);
    /// Get the contents of the clipboard
    fn get_str(&mut self) -> Option<String>;
}

/// The system clipboard provider using arboard crate
pub struct ArboardProvider {
    inner: Mutex<arboard::Clipboard>,
}

impl ClipboardProvider for ArboardProvider {
    fn set_str(&mut self, v: &str) {
        self.inner
            .lock()
            .unwrap()
            .set_text(v)
            .expect("Failed to set clipboard text");
    }
    fn get_str(&mut self) -> Option<String> {
        self.inner.lock().unwrap().get_text().ok()
    }
}

/// Dynamically dispatched clipboard, that avoids doubling the data copied by it by returning an `Rc<str>`
pub struct Clipboard {
    copied_val: Option<(Rc<str>, u64)>,
    inner: Box<dyn ClipboardProvider + Send + Sync>,
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cliboard, copied: {:?}", self.copied_val)
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            copied_val: None,
            inner: Box::new(ArboardProvider {
                inner: Mutex::new(
                    arboard::Clipboard::new().expect("Failed to initialize clipboard"),
                ),
            }),
        }
    }
}

impl Clipboard {
    pub fn with_provider(p: impl ClipboardProvider + Send + Sync + 'static) -> Self {
        Self {
            copied_val: None,
            inner: Box::new(p),
        }
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&mut self, v: Rc<str>) {
        let hash = {
            let mut hasher: hash::DefaultHasher = DefaultHasher::new();
            v.hash(&mut hasher);
            hasher.finish()
        };
        self.inner.set_str(&v);
        self.copied_val = Some((v, hash));
    }
    pub fn get(&mut self) -> Option<Rc<str>> {
        let cb_text = self.inner.get_str()?;
        let cb_hash = {
            let mut hasher: hash::DefaultHasher = DefaultHasher::new();
            cb_text.hash(&mut hasher);
            hasher.finish()
        };
        if let Some((copied, hash)) = self.copied_val.as_ref()
            && *hash == cb_hash
            && copied.as_ref() == cb_text.as_str()
        {
            Some(copied.clone())
        } else {
            let text: Rc<str> = cb_text.into();
            self.copied_val = Some((text.clone(), cb_hash));
            Some(text)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn it_works() {
        let mut clipboard = Clipboard::new();
        let text: &'static str = "Some text";
        clipboard.set(Rc::from(text));
        assert_eq!(clipboard.get().unwrap().as_ref(), text);
    }
}
