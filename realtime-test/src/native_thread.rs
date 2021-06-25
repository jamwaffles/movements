use libc::{
    c_void, mlockall, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create,
    pthread_getschedparam, pthread_join, sched_param, MCL_CURRENT, MCL_FUTURE, PTHREAD_STACK_MIN,
};
use std::{any::Any, cell::UnsafeCell, cmp, ffi::CStr, io, mem, ptr, sync::Arc, time::Duration};

use crate::{InheritPolicy, SchedPolicy};

pub struct Thread {
    id: libc::pthread_t,
}

// Some platforms may have pthread_t as a pointer in which case we still want
// a thread to be Send/Sync
unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(
        stack: usize,
        sched_policy: SchedPolicy,
        p: Box<dyn FnOnce()>,
    ) -> io::Result<Thread> {
        let p = Box::into_raw(Box::new(p));

        let mut param: libc::sched_param = mem::zeroed();
        let mut native: libc::pthread_t = mem::zeroed();
        let mut attr: libc::pthread_attr_t = mem::zeroed();

        // Initialise attributes
        assert_eq!(libc::pthread_attr_init(&mut attr), 0);

        // let stack_size = cmp::max(stack, min_stack_size(&attr));
        let stack_size = cmp::max(stack, PTHREAD_STACK_MIN);

        match libc::pthread_attr_setstacksize(&mut attr, stack_size) {
            0 => {}
            n => {
                assert_eq!(n, libc::EINVAL);

                // // EINVAL means |stack_size| is either too small or not a
                // // multiple of the system page size.  Because it's definitely
                // // >= PTHREAD_STACK_MIN, it must be an alignment issue.
                // // Round up to the nearest page and try again.
                // let page_size = os::page_size();
                // NOTE: Default Linux page size is 4096
                // FIXME: Make dynamic
                let page_size = 4096;
                let stack_size =
                    (stack_size + page_size - 1) & (-(page_size as isize - 1) as usize - 1);

                assert_eq!(libc::pthread_attr_setstacksize(&mut attr, stack_size), 0);
            }
        };

        {
            // Lock memory
            assert_eq!(mlockall(MCL_CURRENT | MCL_FUTURE), 0, "mlockall failed");

            // Set scheduler policy and priority of pthread
            assert_eq!(
                pthread_attr_setschedpolicy(&mut attr, sched_policy as i32),
                0,
                "pthread setschedpolicy failed"
            );

            // TODO: Configurable prio
            // param.sched_priority = 80;
            assert_eq!(
                pthread_attr_setschedparam(&mut attr, &mut param),
                0,
                "pthread setschedparam failed"
            );

            // Use scheduling parameters of attr
            assert_eq!(
                pthread_attr_setinheritsched(&mut attr, InheritPolicy::Explicit as i32),
                0,
                "pthread setinheritsched failed"
            );
        }

        let ret = libc::pthread_create(&mut native, &attr, thread_start, p as *mut _);
        // Note: if the thread creation fails and this assert fails, then p will
        // be leaked. However, an alternative design could cause double-free
        // which is clearly worse.
        assert_eq!(libc::pthread_attr_destroy(&mut attr), 0);

        return if ret != 0 {
            // The thread failed to start and as a result p was not consumed. Therefore, it is
            // safe to reconstruct the box so that it gets deallocated.
            drop(Box::from_raw(p));
            Err(io::Error::from_raw_os_error(ret))
        } else {
            Ok(Thread { id: native })
        };

        extern "C" fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
            unsafe {
                // Next, set up our stack overflow handler which may get triggered if we run
                // out of stack.
                // FIXME
                // let _handler = stack_overflow::Handler::new();

                // Finally, let's run some code.
                Box::from_raw(main as *mut Box<dyn FnOnce()>)();
            }
            ptr::null_mut()
        }
    }

    pub fn yield_now() {
        let ret = unsafe { libc::sched_yield() };
        debug_assert_eq!(ret, 0);
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn set_name(name: &CStr) {
        const PR_SET_NAME: libc::c_int = 15;
        // pthread wrapper only appeared in glibc 2.12, so we use syscall
        // directly.
        unsafe {
            libc::prctl(PR_SET_NAME, name.as_ptr() as libc::c_ulong, 0, 0, 0);
        }
    }

    pub fn sleep(dur: Duration) {
        let mut secs = dur.as_secs();
        let mut nsecs = dur.subsec_nanos() as _;

        // If we're awoken with a signal then the return value will be -1 and
        // nanosleep will fill in `ts` with the remaining time.
        unsafe {
            while secs > 0 || nsecs > 0 {
                let mut ts = libc::timespec {
                    tv_sec: cmp::min(libc::time_t::MAX as u64, secs) as libc::time_t,
                    tv_nsec: nsecs,
                };
                secs -= ts.tv_sec as u64;
                let ts_ptr = &mut ts as *mut _;
                if libc::nanosleep(ts_ptr, ts_ptr) == -1 {
                    // FIXME
                    // assert_eq!(os::errno(), libc::EINTR);
                    secs += ts.tv_sec as u64;
                    nsecs = ts.tv_nsec;
                } else {
                    nsecs = 0;
                }
            }
        }
    }

    pub fn join(self) {
        unsafe {
            let ret = libc::pthread_join(self.id, ptr::null_mut());
            mem::forget(self);
            assert!(
                ret == 0,
                "failed to join thread: {}",
                io::Error::from_raw_os_error(ret)
            );
        }
    }

    pub fn id(&self) -> libc::pthread_t {
        self.id
    }

    pub fn into_id(self) -> libc::pthread_t {
        let id = self.id;
        mem::forget(self);
        id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let ret = unsafe { libc::pthread_detach(self.id) };
        debug_assert_eq!(ret, 0);
    }
}

// From https://github.com/mahkoh/posix.rs/blob/master/src/pthread/mod.rs
pub fn pthread_attr_setinheritsched(
    attr: &mut pthread_attr_t,
    inherit: libc::c_int,
) -> libc::c_int {
    extern "C" {
        fn pthread_attr_setinheritsched(
            attr: *mut pthread_attr_t,
            inherit: libc::c_int,
        ) -> libc::c_int;
    }
    unsafe { pthread_attr_setinheritsched(attr as *mut _, inherit) }
}

pub fn pthread_attr_setschedparam(attr: &mut pthread_attr_t, param: &sched_param) -> libc::c_int {
    extern "C" {
        fn pthread_attr_setschedparam(
            attr: *mut pthread_attr_t,
            param: *const sched_param,
        ) -> libc::c_int;
    }
    unsafe { pthread_attr_setschedparam(attr as *mut _, param as *const _) }
}

pub fn pthread_attr_setschedpolicy(attr: &mut pthread_attr_t, policy: libc::c_int) -> libc::c_int {
    extern "C" {
        fn pthread_attr_setschedpolicy(
            attr: *mut pthread_attr_t,
            policy: libc::c_int,
        ) -> libc::c_int;
    }
    unsafe { pthread_attr_setschedpolicy(attr as *mut _, policy) }
}
