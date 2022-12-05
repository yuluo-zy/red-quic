use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use parking_lot::Mutex;

#[derive(Clone)]
pub struct IsClosed {
    is_closed: Arc<AtomicBool>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl IsClosed {
    pub fn new() -> Self {
        Self {
            is_closed: Arc::new(AtomicBool::new(false)),
            waker: Arc::new(Mutex::new(None)),
        }
    }
    pub fn set_close(&self) {
        self.is_closed.store(true, Ordering::Release);
        if let Some(waker) = self.waker.lock().take() {
            waker.wake();
        }
    }

    fn check(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }
}

impl Future for IsClosed {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_closed.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            *self.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct IsAuth {
    is_close: IsClosed,
    is_auth: Arc<AtomicBool>,
    waker: Arc<Mutex<Vec<Waker>>>,
}

impl IsAuth {
    pub fn new(is_close: IsClosed) -> Self {
        Self {
            is_close,
            is_auth: Arc::new(AtomicBool::new(false)),
            waker: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_auth(&self) {
        self.is_auth.store(true, Ordering::Release);
    }

    pub fn wake(&self) {
        // 批量唤醒
        for item in self.waker.lock().drain(..) {
            item.wake()
        }
    }
}

impl Future for IsAuth {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_close.check() {
            Poll::Ready(false)
        } else if self.is_auth.load(Ordering::Relaxed) {
            Poll::Ready(true)
        } else {
            self.waker.lock().push(cx.waker().clone());
            Poll::Pending
        }
    }
}

