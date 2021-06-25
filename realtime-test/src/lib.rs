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
// type ThreadResult<T> = Result<T, Box<dyn Any + Send + 'static>>;

// // TODO: Return value
// pub fn spawn_with_policy<'a, F, T>(func: F, policy: SchedPolicy) -> JoinHandle<T>
// where
//     F: FnOnce() -> T,
//     F: Send + 'a,
//     T: Send + 'a,
// {
//     unsafe {
//         let mut param: libc::sched_param = mem::zeroed();
//         let mut thread: libc::pthread_t = mem::zeroed();
//         let mut attr: libc::pthread_attr_t = mem::zeroed();

//         // // Yes, this does appear to need a double box. What.
//         // let func: Box<dyn FnOnce()> = Box::new(func);
//         // let func = Box::into_raw(Box::new(func));

//         // Lock memory
//         if mlockall(MCL_CURRENT | MCL_FUTURE) == -1 {
//             println!("mlockall failed: %m\n");
//             exit(-2);
//         }

//         // Initialize pthread attributes (default values)
//         let ret = pthread_attr_init(&mut attr);
//         assert_eq!(ret, 0, "init pthread attributes failed");

//         // Set a specific stack size
//         let ret = pthread_attr_setstacksize(&mut attr, PTHREAD_STACK_MIN);
//         assert_eq!(ret, 0, "pthread setstacksize failed");

//         // Set scheduler policy and priority of pthread
//         let ret = pthread_attr_setschedpolicy(&mut attr, policy as i32);
//         assert_eq!(ret, 0, "pthread setschedpolicy failed");

//         // TODO: Configurable prio
//         // param.sched_priority = 80;
//         let ret = pthread_attr_setschedparam(&mut attr, &mut param);
//         assert_eq!(ret, 0, "pthread setschedparam failed");

//         // Use scheduling parameters of attr
//         let ret = pthread_attr_setinheritsched(&mut attr, InheritPolicy::Explicit as i32);
//         assert_eq!(ret, 0, "pthread setinheritsched failed");

//         let my_packet: Arc<UnsafeCell<Option<ThreadResult<T>>>> = Arc::new(UnsafeCell::new(None));
//         let their_packet = my_packet.clone();

//         let main = move || {
//             let try_result = panic::catch_unwind(panic::AssertUnwindSafe(|| func()));

//             // SAFETY: `their_packet` as been built just above and moved by the
//             // closure (it is an Arc<...>) and `my_packet` will be stored in the
//             // same `JoinInner` as this closure meaning the mutation will be
//             // safe (not modify it and affect a value far away).
//             *their_packet.get() = Some(try_result);
//         };

//         let main = Box::new(main);

//         // Create a pthread with specified attributes
//         let ret = pthread_create(
//             &mut thread,
//             &mut attr,
//             thread_start,
//             Box::into_raw(mem::transmute::<
//                 Box<dyn FnOnce() + 'a>,
//                 Box<dyn FnOnce() + 'static>,
//             >(Box::new(main))) as *mut _,
//         );
//         assert_eq!(ret, 0, "create pthread failed");

//         extern "C" fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
//             unsafe {
//                 // Next, set up our stack overflow handler which may get triggered if we run
//                 // out of stack.
//                 // FIXME
//                 // let _handler = stack_overflow::Handler::new();
//                 // Finally, let's run some code.
//                 Box::from_raw(main as *mut Box<dyn FnOnce()>)();
//             }
//             ptr::null_mut()
//         }

//         // TODO: `pthread_attr_destroy`

//         // Just some debug
//         {
//             let mut new_policy = mem::zeroed();
//             let mut new_param = mem::zeroed();

//             let ret = pthread_getschedparam(thread, &mut new_policy, &mut new_param);
//             println!(
//                 "Sched policy: {:?} {:?} {:?}",
//                 ret, new_policy, new_param.sched_priority
//             );
//         }

//         JoinHandle(JoinInner {
//             native_thread_id: thread,
//             packet: Packet(my_packet),
//         })
//     }
// }
