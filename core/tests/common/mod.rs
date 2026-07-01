#![allow(dead_code)]

use flexweave::{DataStore, ObjectId, ObjectStore};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestAtom {
    Ability,
    Burst,
    Category,
    Group,
    Family,
    Variant,
}

pub fn marker_setup() -> (ObjectStore, DataStore<u8>, ObjectId, ObjectId, ObjectId) {
    let mut objects = ObjectStore::new();
    let mut markers = DataStore::new();

    let excluded = objects.create();
    markers.attach(excluded, 1);

    let first_match = objects.create();
    markers.attach(first_match, 1);

    let non_match = objects.create();
    markers.attach(non_match, 2);

    let second_match = objects.create();
    markers.attach(second_match, 1);

    let no_data = objects.create();
    let _ = no_data;

    let third_match = objects.create();
    markers.attach(third_match, 1);

    (objects, markers, excluded, first_match, second_match)
}

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = Pin::from(Box::new(future));
    loop {
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
