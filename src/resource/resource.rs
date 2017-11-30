use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::borrow::Borrow;
use std;

use two_lock_queue;
use futures;
use futures::Future;

use super::{ResourceFuture, ResourceArenaLoader, ResourceArenaMapper};
use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

/// The centralized resource management system.
pub struct ResourceSystem {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    /// Creates a new `ResourceSystem`.
    ///
    /// Notes that this will spawn a worker thread running background to perform
    /// io requests.
    pub fn new() -> Result<Self> {
        let driver = Arc::new(RwLock::new(FilesystemDriver::new()));

        let (tx, rx) = two_lock_queue::channel(1024);

        {
            let driver = driver.clone();
            thread::spawn(|| { ResourceSystem::run(rx, driver); });
        }

        let shared = ResourceSystemShared::new(driver.clone(), tx);

        Ok(ResourceSystem {
               filesystems: driver,
               shared: Arc::new(shared),
           })
    }

    /// Returns the shared parts of `ResourceSystem`.
    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }

    /// Mount a file-system drive with identifier.
    #[inline]
    pub fn mount<S, F>(&self, ident: S, fs: F) -> Result<()>
        where S: Borrow<str>,
              F: Filesystem + 'static
    {
        self.filesystems.write().unwrap().mount(ident, fs)
    }

    /// Unmount a file-system from this collection.
    #[inline]
    pub fn unmount<S>(&self, ident: S)
        where S: Borrow<str>
    {
        self.filesystems.write().unwrap().unmount(ident);
    }

    fn run(chan: two_lock_queue::Receiver<ResourceTask>, driver: Arc<RwLock<FilesystemDriver>>) {
        let mut buf = Vec::new();

        loop {
            match chan.recv().unwrap() {
                ResourceTask::Load { mut closure } => {
                    let driver = driver.read().unwrap();
                    closure(&driver, &mut buf);
                }

                ResourceTask::Map { mut closure } => {
                    closure();
                }

                ResourceTask::Stop => return,
            }
        }
    }

    fn load<T>(slave: &T,
               path: &Path,
               driver: &FilesystemDriver,
               buf: &mut Vec<u8>)
               -> std::result::Result<T::Item, T::Error>
        where T: ResourceArenaLoader
    {
        if let Some(v) = slave.get(&path) {
            return Ok(v);
        }

        let from = buf.len();
        driver.load_into(&path, buf)?;
        let asset = slave.insert(&path, &buf[from..])?;
        Ok(asset)
    }
}

pub struct ResourceSystemShared {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    chan: two_lock_queue::Sender<ResourceTask>,
}

enum ResourceTask {
    Load { closure: Box<FnMut(&FilesystemDriver, &mut Vec<u8>) + Send + Sync>, },
    Map { closure: Box<FnMut() + Send + Sync> },
    Stop,
}

impl ResourceSystemShared {
    fn new(filesystems: Arc<RwLock<FilesystemDriver>>,
           chan: two_lock_queue::Sender<ResourceTask>)
           -> Self {
        ResourceSystemShared {
            filesystems: filesystems,
            chan: chan,
        }
    }

    pub fn exists<T, P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        self.filesystems.read().unwrap().exists(path)
    }

    pub fn load<T, P>(&self, slave: T, path: P) -> ResourceFuture<T::Item, T::Error>
        where T: ResourceArenaLoader,
              P: AsRef<Path>
    {
        let (tx, rx) = futures::sync::oneshot::channel();

        // Returns directly if we have this resource in memory.
        if let Some(v) = slave.get(path.as_ref()) {
            tx.send(Ok(v)).is_ok();
            return ResourceFuture(rx);
        }

        // Hacks: Optimize this when Box<FnOnce> is usable.
        let path = path.as_ref().to_owned();
        let payload = Arc::new(RwLock::new(Some((tx, path, slave))));
        let closure = move |d: &FilesystemDriver, b: &mut Vec<u8>| if let Some(data) =
            payload.write().unwrap().take() {
            let v = ResourceSystem::load::<T>(&data.2, &data.1, d, b);
            data.0.send(v).is_ok();
        };

        self.chan
            .send(ResourceTask::Load { closure: Box::new(closure) })
            .unwrap();

        ResourceFuture(rx)
    }

    pub fn map<T>(&self,
                  slave: T,
                  src: ResourceFuture<T::Source, T::Error>)
                  -> ResourceFuture<T::Item, T::Error>
        where T: ResourceArenaMapper
    {
        let (tx, rx) = futures::sync::oneshot::channel();

        // Hacks: Optimize this when Box<FnOnce> is usable.
        let payload = Arc::new(RwLock::new(Some((tx, slave, src))));
        let closure = move || if let Some(mut data) = payload.write().unwrap().take() {
            let v = match data.2.poll() {
                Err(err) => Err(err),
                Ok(task) => {
                    match task {
                        futures::Async::Ready(v) => data.1.map(&v),
                        futures::Async::NotReady => {
                            unreachable!("Trying to map from unknown source.")
                        }
                    }
                }
            };

            data.0.send(v).is_ok();
        };

        self.chan
            .send(ResourceTask::Map { closure: Box::new(closure) })
            .unwrap();

        ResourceFuture(rx)
    }
}

impl Drop for ResourceSystemShared {
    fn drop(&mut self) {
        self.chan.send(ResourceTask::Stop).unwrap();
    }
}