#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(test)]
use std::ffi::{OsStr, OsString};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(test)]
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(crate) struct TestEnvGuard {
    _lock: MutexGuard<'static, ()>,
    original: BTreeMap<String, Option<OsString>>,
}

#[cfg(test)]
impl TestEnvGuard {
    pub(crate) fn set<const N: usize>(entries: [(&str, Option<&str>); N]) -> Self {
        let lock = env_lock().lock().expect("lock test env mutex");
        let mut original = BTreeMap::new();
        for (key, _) in entries.iter().copied() {
            original
                .entry(key.to_string())
                .or_insert_with(|| std::env::var_os(key));
        }

        for (key, value) in entries {
            match value {
                Some(value) => set_env_var(key, value),
                None => remove_env_var(key),
            }
        }

        Self {
            _lock: lock,
            original,
        }
    }
}

#[cfg(test)]
impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.original {
            match value {
                Some(value) => set_env_var(key, value),
                None => remove_env_var(key),
            }
        }
    }
}

#[cfg(test)]
fn set_env_var(key: &str, value: impl AsRef<OsStr>) {
    // SAFETY: all test-time environment mutation goes through the shared
    // `env_lock`, so no concurrent unit test in this process can race on
    // process-global environment state.
    unsafe {
        std::env::set_var(key, value);
    }
}

#[cfg(test)]
fn remove_env_var(key: &str) {
    // SAFETY: all test-time environment mutation goes through the shared
    // `env_lock`, so removing the variable cannot race with another unit
    // test in this process.
    unsafe {
        std::env::remove_var(key);
    }
}
