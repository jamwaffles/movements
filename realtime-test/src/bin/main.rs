use std::{mem, process::exit, ptr};

use libc::{
    c_void, mlockall, posix_spawnattr_setschedparam, posix_spawnattr_setschedpolicy,
    pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create,
    pthread_getschedparam, pthread_join, MCL_CURRENT, MCL_FUTURE, PTHREAD_STACK_MIN, SCHED_FIFO,
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

// Just a guess
const PTHREAD_INHERIT_SCHED: usize = 0;
const PTHREAD_EXPLICIT_SCHED: usize = 1;

// TODO: Enum `inheritsched`
extern "C" {
    fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: usize) -> libc::c_int;
}

fn main() {
    let res = unsafe {
        let mut param: libc::sched_param = mem::zeroed();
        let mut thread: libc::pthread_t = mem::zeroed();
        let mut attr: libc::pthread_attr_t = mem::zeroed();
        // let mut spawn_attr: libc::posix_spawnattr_t = mem::zeroed();

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
        let ret = posix_spawnattr_setschedpolicy(
            &mut attr as *mut _ as *mut libc::posix_spawnattr_t,
            SCHED_FIFO,
        );
        println!("C");
        if ret != 0 {
            println!("pthread setschedpolicy failed\n");
            return;
        }
        param.sched_priority = 80;
        let ret = posix_spawnattr_setschedparam(
            &mut attr as *mut _ as *mut libc::posix_spawnattr_t,
            &param,
        );
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
