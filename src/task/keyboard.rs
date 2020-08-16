use crate::misc::Once;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

const SC_CAP: usize = 100;
/// Holds scancodes added by `add_scancode`.
static SCANCODES: Once<ArrayQueue<u8>> = Once::new();
static SCANCODES_WAKER: AtomicWaker = AtomicWaker::new();

static DECODED_KEYS: Once<ArrayQueue<DecodedKey>> = Once::new();
static DECODED_KEYS_WAKER: AtomicWaker = AtomicWaker::new();

fn clear_array_queue<T>(queue: &ArrayQueue<T>) {
    while queue.pop().is_ok() {}
}

/// Handles scancodes asynchronously.
pub async fn handle_scancodes() {
    DECODED_KEYS.try_init(ArrayQueue::new(SC_CAP));

    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                add_decoded_key(key);
            }
        }
    }
}

/// Called by the keyboard interrupt handler.
///
/// Must not block or allocate.
pub fn add_scancode(scancode: u8) {
    use log::warn;

    if let Some(queue) = SCANCODES.get() {
        if queue.push(scancode).is_err() {
            warn!("scancode queue full, clearing queue to avoid dropping keyboard input");
            clear_array_queue(&queue);
        } else {
            SCANCODES_WAKER.wake();
        }
    } else {
        warn!("scancode queue uninitialized");
    }
}

/// Called by the scancode handler function.
///
/// Must not block or allocate.
fn add_decoded_key(key: DecodedKey) {
    use log::warn;
    if let Some(d) = DECODED_KEYS.get() {
        if d.push(key).is_err() {
            warn!("decoded key queue full");
        } else {
            DECODED_KEYS_WAKER.wake();
        }
    } else {
        warn!("decoded key queue uninitialized");
    }
}

/// Polls scancodes from `SCANCODE_QUEUE`.
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODES.try_init(ArrayQueue::new(SC_CAP));
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODES.get().expect("not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        SCANCODES_WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                SCANCODES_WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

/// Polls `DecodedKey`s from `DECODED_KEYS`.
pub struct DecodedKeyStream;

impl Stream for DecodedKeyStream {
    type Item = DecodedKey;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = if let Some(q) = DECODED_KEYS.get() {
            q
        } else {
            return Poll::Pending;
        };

        if let Ok(key) = queue.pop() {
            return Poll::Ready(Some(key));
        }

        DECODED_KEYS_WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(key) => {
                DECODED_KEYS_WAKER.take();
                Poll::Ready(Some(key))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}
