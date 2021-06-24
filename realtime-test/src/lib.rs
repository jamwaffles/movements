use std::{cmp, io, mem, ptr};

// pub mod thread;

pub struct Thread {
    id: u64,
}

impl Thread {
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
}

pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024;

// Default on Linux
pub const PAGE_SIZE: usize = 4096;

pub unsafe fn spawn(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
    let p = Box::into_raw(p);
    let mut native: libc::pthread_t = mem::zeroed();
    let mut attr: libc::pthread_attr_t = mem::zeroed();
    assert_eq!(libc::pthread_attr_init(&mut attr), 0);

    let stack_size = cmp::max(stack, DEFAULT_MIN_STACK_SIZE);

    println!("A");

    match libc::pthread_attr_setstacksize(&mut attr, stack_size) {
        0 => {}
        n => {
            assert_eq!(n, libc::EINVAL, "assert 1");
            // EINVAL means |stack_size| is either too small or not a
            // multiple of the system page size.  Because it's definitely
            // >= PTHREAD_STACK_MIN, it must be an alignment issue.
            // Round up to the nearest page and try again.
            let page_size = PAGE_SIZE;
            let stack_size =
                (stack_size + page_size - 1) & (-(page_size as isize - 1) as usize - 1);
            assert_eq!(libc::pthread_attr_setstacksize(&mut attr, stack_size), 0);
        }
    };

    println!("B");

    let ret = libc::pthread_create(&mut native, &attr, thread_start, p as *mut _);
    // Note: if the thread creation fails and this assert fails, then p will
    // be leaked. However, an alternative design could cause double-free
    // which is clearly worse.
    assert_eq!(libc::pthread_attr_destroy(&mut attr), 0, "assert 2");

    println!("C");

    return if ret != 0 {
        // The thread failed to start and as a result p was not consumed. Therefore, it is
        // safe to reconstruct the box so that it gets deallocated.
        drop(Box::from_raw(p));
        Err(io::Error::from_raw_os_error(ret))
    } else {
        println!("D");
        Ok(Thread { id: native })
    };

    extern "C" fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
        println!("E");
        unsafe {
            //     // Next, set up our stack overflow handler which may get triggered if we run
            //     // out of stack.
            //     // let _handler = Handler::new();
            //     // Finally, let's run some code.
            Box::from_raw(main as *mut Box<dyn FnOnce()>)();
        }
        ptr::null_mut()
    }
}
