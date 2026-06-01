use std::{
    future::Future,
    sync::mpsc::{Receiver, SyncSender, sync_channel},
    thread::{self, JoinHandle},
};

pub(super) fn lifecycle_reply_channel()
-> (SyncSender<Result<(), String>>, Receiver<Result<(), String>>) {
    sync_channel(1)
}

pub(super) fn spawn_zeromq_worker<F>(name: &str, future: F) -> JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("zeromq worker runtime should build");
            runtime.block_on(future);
        })
        .expect("zeromq worker thread should spawn")
}
