use std::{mem, process::exit, ptr};

use libc::{
    c_void, mlockall, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create,
    pthread_getschedparam, pthread_join, sched_param, MCL_CURRENT, MCL_FUTURE, PTHREAD_STACK_MIN,
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

/// Values from https://github.com/mahkoh/posix.rs/blob/master/src/pthread/linux/x86_64.rs
pub enum InheritPolicy {
    Inherit = 0,
    Explicit = 1,
}

pub enum SchedPolicy {
    Other = 0,
    Fifo = 1,
    Rr = 2,
    Batch = 3,
    Idle = 5,
}

pub struct JoinHandle {
    native_thread_id: u64,
}

impl JoinHandle {
    pub fn join(self) -> Result<(), i32> {
        let ret =
            unsafe { pthread_join(self.native_thread_id, ptr::null_mut() as *mut *mut c_void) };

        if ret != 0 {
            Err(ret)
        } else {
            Ok(())
        }
    }
}

// TODO: Return value
pub fn spawn_with_policy<F>(func: F, policy: SchedPolicy) -> JoinHandle
where
    F: FnOnce() -> (),
    F: Send + 'static,
{
    unsafe {
        let mut param: libc::sched_param = mem::zeroed();
        let mut thread: libc::pthread_t = mem::zeroed();
        let mut attr: libc::pthread_attr_t = mem::zeroed();

        // Yes, this does appear to need a double box. What.
        let func: Box<dyn FnOnce()> = Box::new(func);
        let func = Box::into_raw(Box::new(func));

        // Lock memory
        if mlockall(MCL_CURRENT | MCL_FUTURE) == -1 {
            println!("mlockall failed: %m\n");
            exit(-2);
        }

        // Initialize pthread attributes (default values)
        let ret = pthread_attr_init(&mut attr);
        assert_eq!(ret, 0, "init pthread attributes failed");

        // Set a specific stack size
        let ret = pthread_attr_setstacksize(&mut attr, PTHREAD_STACK_MIN);
        assert_eq!(ret, 0, "pthread setstacksize failed");

        // Set scheduler policy and priority of pthread
        let ret = pthread_attr_setschedpolicy(&mut attr, policy as i32);
        assert_eq!(ret, 0, "pthread setschedpolicy failed");

        // TODO: Configurable prio
        // param.sched_priority = 80;
        let ret = pthread_attr_setschedparam(&mut attr, &mut param);
        assert_eq!(ret, 0, "pthread setschedparam failed");

        // Use scheduling parameters of attr
        let ret = pthread_attr_setinheritsched(&mut attr, InheritPolicy::Explicit as i32);
        assert_eq!(ret, 0, "pthread setinheritsched failed");

        // Create a pthread with specified attributes
        let ret = pthread_create(&mut thread, &mut attr, thread_start, func as *mut _);
        assert_eq!(ret, 0, "create pthread failed");

        // TODO: `pthread_attr_destroy`

        // Just some debug
        {
            let mut new_policy = mem::zeroed();
            let mut new_param = mem::zeroed();

            let ret = pthread_getschedparam(thread, &mut new_policy, &mut new_param);
            println!(
                "Sched policy: {:?} {:?} {:?}",
                ret, new_policy, new_param.sched_priority
            );
        }

        JoinHandle {
            native_thread_id: thread,
        }
    }
}
