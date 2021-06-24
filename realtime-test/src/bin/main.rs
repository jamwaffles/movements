use std::{mem, process::exit, ptr};

use libc::{
    c_void, mlockall, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create,
    pthread_getschedparam, pthread_join, sched_param, MCL_CURRENT, MCL_FUTURE, PTHREAD_STACK_MIN,
    SCHED_FIFO,
};

extern "C" fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
    unsafe {
        // Next, set up our stack overflow handler which may get triggered if we run
        // out of stack.
        // let _handler = stack_overflow::Handler::new();
        // Finally, let's run some code.
        Box::from_raw(main as *mut Box<dyn FnOnce()>)();
    }
    ptr::null_mut()
}

// Values from https://github.com/mahkoh/posix.rs/blob/master/src/pthread/linux/x86_64.rs
const PTHREAD_INHERIT_SCHED: i32 = 0;
const PTHREAD_EXPLICIT_SCHED: i32 = 1;

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

fn main() {
    let res = unsafe {
        let mut param: libc::sched_param = mem::zeroed();
        let mut thread: libc::pthread_t = mem::zeroed();
        let mut attr: libc::pthread_attr_t = mem::zeroed();
        // let mut spawn_attr: libc::posix_attr_t = mem::zeroed();

        let func = move || {
            println!("Thread");
        };

        // Yes, this does appear to need a double box. What.
        let func: Box<dyn FnOnce()> = Box::new(func);
        let func = Box::into_raw(Box::new(func));

        /* Lock memory */
        if mlockall(MCL_CURRENT | MCL_FUTURE) == -1 {
            println!("mlockall failed: %m\n");
            exit(-2);
        }

        /* Initialize pthread attributes (default values) */
        let ret = pthread_attr_init(&mut attr);
        println!("A");
        if ret != 0 {
            println!("init pthread attributes failed\n");
            return;
        }

        /* Set a specific stack size  */
        let ret = pthread_attr_setstacksize(&mut attr, PTHREAD_STACK_MIN);
        println!("B");
        if ret != 0 {
            println!("pthread setstacksize failed\n");
            return;
        }

        /* Set scheduler policy and priority of pthread */
        let ret = pthread_attr_setschedpolicy(&mut attr, SCHED_FIFO);
        println!("C");
        if ret != 0 {
            println!("pthread setschedpolicy failed\n");
            return;
        }
        param.sched_priority = 80;
        let ret = pthread_attr_setschedparam(&mut attr, &param);
        println!("D");
        if ret != 0 {
            println!("pthread setschedparam failed\n");
            return;
        }

        /* Use scheduling parameters of attr */
        let ret = pthread_attr_setinheritsched(&mut attr, PTHREAD_EXPLICIT_SCHED);
        if ret != 0 {
            println!("pthread setinheritsched failed\n");
            return;
        }

        /* Create a pthread with specified attributes */
        let ret = pthread_create(&mut thread, &mut attr, thread_start, func as *mut _);
        println!("E");
        if ret != 0 {
            println!("create pthread failed\n");
            return;
        }

        let mut new_policy = mem::zeroed();
        let mut new_param = mem::zeroed();

        let ret = pthread_getschedparam(thread, &mut new_policy, &mut new_param);
        println!(
            "Sched policy: {:?} {:?} {:?}",
            ret, new_policy, new_param.sched_priority
        );

        /* Join the thread and wait until it is done */
        let ret = pthread_join(thread, ptr::null_mut() as *mut *mut c_void);
        if ret != 0 {
            println!("join pthread failed: %m\n");
        }

        println!("F");

        0
    };

    dbg!(res);
}
