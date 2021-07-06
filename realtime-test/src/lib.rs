mod join_handle;
mod native_thread;

use join_handle::{JoinHandle, JoinInner, Packet};
use libc::PTHREAD_STACK_MIN;
use std::{cell::UnsafeCell, fmt::Display, io, mem, panic, str::FromStr, sync::Arc};

/// Values from https://github.com/mahkoh/posix.rs/blob/master/src/pthread/linux/x86_64.rs
#[derive(Debug, Copy, Clone)]
pub enum InheritPolicy {
    Inherit = 0,
    Explicit = 1,
}

// TODO: Add priority calculations/ranges to each variant
#[derive(Debug, Copy, Clone)]
pub enum SchedPolicy {
    /// Standard round robin. Priority 0.
    Other = 0,
    // /// Batch processing. Priority 0.
    // Batch = 3,
    // /// For super low priority background tasks. Priority 0.
    // Idle = 5,
    /// Realtime, FIFO. Priority 1 - 99.
    Fifo = 1,
    // Realtime, Round Robin. Priority 1 - 99.
    Rr = 2,
}

impl FromStr for SchedPolicy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "other" => Ok(Self::Other),
            // "batch" => Ok(Self::Batch),
            // "idle" => Ok(Self::Idle),
            "fifo" => Ok(Self::Fifo),
            "rr" => Ok(Self::Rr),
            _ => Err(()),
        }
    }
}

impl Display for SchedPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other => f.write_str("other"),
            // Self::Batch => f.write_str("batch"),
            // Self::Idle => f.write_str("idle"),
            Self::Fifo => f.write_str("fifo"),
            Self::Rr => f.write_str("rr"),
        }
    }
}

pub fn spawn_unchecked<'a, F, T>(
    sched_policy: SchedPolicy,
    priority: i32,
    f: F,
) -> io::Result<JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: Send + 'a,
    T: Send + 'a,
{
    // let stack_size = stack_size.unwrap_or_else(thread::min_stack);

    // let my_thread = Thread::new(name);
    // let their_thread = my_thread.clone();

    let my_packet: Arc<UnsafeCell<Option<crate::join_handle::Result<T>>>> =
        Arc::new(UnsafeCell::new(None));
    let their_packet = my_packet.clone();

    // let output_capture = crate::io::set_output_capture(None);
    // crate::io::set_output_capture(output_capture.clone());

    let main = move || {
        // if let Some(name) = their_thread.cname() {
        //     imp::Thread::set_name(name);
        // }

        // crate::io::set_output_capture(output_capture);

        // SAFETY: the stack guard passed is the one for the current thread.
        // This means the current thread's stack and the new thread's stack
        // are properly set and protected from each other.
        // FIXME
        // thread_info::set(unsafe { imp::guard::current() }, their_thread);

        let try_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            // crate::sys_common::backtrace::__rust_begin_short_backtrace(f)
            f()
        }));
        // SAFETY: `their_packet` as been built just above and moved by the
        // closure (it is an Arc<...>) and `my_packet` will be stored in the
        // same `JoinInner` as this closure meaning the mutation will be
        // safe (not modify it and affect a value far away).
        unsafe { *their_packet.get() = Some(try_result) };
    };

    Ok(JoinHandle(JoinInner {
        // SAFETY:
        //
        // `imp::Thread::new` takes a closure with a `'static` lifetime, since it's passed
        // through FFI or otherwise used with low-level threading primitives that have no
        // notion of or way to enforce lifetimes.
        //
        // As mentioned in the `Safety` section of this function's documentation, the caller of
        // this function needs to guarantee that the passed-in lifetime is sufficiently long
        // for the lifetime of the thread.
        //
        // Similarly, the `sys` implementation must guarantee that no references to the closure
        // exist after the thread has terminated, which is signaled by `Thread::join`
        // returning.
        native: unsafe {
            Some(native_thread::Thread::new(
                PTHREAD_STACK_MIN,
                sched_policy,
                priority,
                mem::transmute::<Box<dyn FnOnce() + 'a>, Box<dyn FnOnce() + 'static>>(Box::new(
                    main,
                )),
            )?)
        },
        // thread: my_thread,
        packet: Packet(my_packet),
    }))
}

pub fn set_thread_prio(prio: i32, policy: SchedPolicy) {
    let mut param: libc::sched_param = unsafe { mem::zeroed() };
    param.sched_priority = prio;

    assert_eq!(
        unsafe { libc::sched_setscheduler(0, policy as i32, &param) },
        0,
        "failed to set prio"
    );
}
