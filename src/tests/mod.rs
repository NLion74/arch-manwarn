mod match_entries_test;

use crate::config::Config;
use simple_semaphore::{Permit, Semaphore};
use std::sync::{Arc, LazyLock};
use utils::ChainList;

/// This is used to dynamically set the configuration.
///
/// Should be used with [init_config]
pub static CONFIG: LazyLock<ChainList<Config>> =
    LazyLock::new(|| ChainList::new(Config::default()));

/// To keep [CONFIG] unmodified, the [Permit] should be hold until the function ends.
/// E.g.
/// ``` no_run
/// fn foo() {
///     let _permit = init_config(config);
///     // do tests
///     drop(_permit); // not necessary
/// }
/// ```
fn init_config(conf: Config) -> Permit {
    static SEMAPHORE: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Semaphore::new(1));
    // ensure config is kept before being overwritten
    let permit = SEMAPHORE.acquire();
    CONFIG.extend(conf);
    permit
}

mod utils {
    use std::{ops::Deref, sync::OnceLock};

    pub struct ChainList<T> {
        inner: T,
        next: OnceLock<Box<ChainList<T>>>,
    }

    impl<T> ChainList<T> {
        pub(super) const fn new(inner: T) -> Self {
            Self {
                inner,
                next: OnceLock::new(),
            }
        }

        pub(super) fn extend(&self, inner: T) {
            if self
                .as_inner_self()
                .next
                .set(Box::new(ChainList {
                    inner,
                    next: OnceLock::new(),
                }))
                .is_err()
            {
                unreachable!("ChainList failed to extend itself")
            }
        }

        fn as_inner_self(&self) -> &Self {
            if let Some(next) = self.next.get() {
                next.as_inner_self()
            } else {
                &self
            }
        }
    }

    impl<T> Deref for ChainList<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.as_inner_self().inner
        }
    }

    #[test]
    fn test() {
        let target = ChainList::new(1u8);
        assert_eq!(*target, 1);
        target.extend(2);
        assert_eq!(*target, 2);
        target.extend(8);
        assert_eq!(*target, 8);
    }
}
