use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use parking_lot::Mutex;

struct IsClosedInner {
    is_closed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

#[derive(Clone)]
pub struct IsClosed(Arc<IsClosedInner>);

impl IsClosed {
    fn new() -> Self {
        Self(Arc::new(IsClosedInner{
            is_closed: AtomicBool::new(false),
            waker: Mutex::new(None),
        }))
    }

    fn set_close(&self) {
        self.0.is_closed.store(true, Ordering::Release);
        if let Some(waker) = self.0.waker.lock().take() {
            waker.wake();
        }
    }

    fn check(&self) ->bool {
        self.0.is_closed.load(Ordering::Acqukire)
    }
}

impl Future for IsClosed {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_closed.load(Ordering::Acquire) {
            Poll::Ready(())
        }else {
            *self.0.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}



pub struct IsAuth {

}