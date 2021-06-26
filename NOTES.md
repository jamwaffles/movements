# Pull parser

Declare `Number` type as `f32` to begin with. Open up possibility of using `fixed` or `f64` later.

## Case 1: No alloc, no std

Use case: 3D printers, simpler machines drip fed from other devices. Unidirectional program flow.

Procedures nor control flow are supported. Variables are supported.

1. Parse input as a bunch of blocks
2. Parse each word from each block into a `Token` struct with position in the original input and a `Word` enum.

Variable assignment is handled in the interpreter/state machine. This could be a fixed array for numeric vars and a `heapless` `Vec` for named variables.

## Case 2: Alloc

Use case: rPi/PC based machine controls with more memory and need/desire to validate/backplot the program.

- Add methods to parse entire program into a tree of `Token`s.
  - Support procedures
  - Support control flow
- Add extra items to `Word` enum, or make a new `ComplexWord` (need a better name) enum with `Word` nested inside it

# Alloc-only: `ProgramTree`

A struct which can consume the pull parser into a token tree, allowing control flow and procedures and stuff.

# Interpreter

Interpreter provides a type param for named variable storage. This would need to implement a trait, but could use a static array (const generics) or a `heapless` `HashMap` (more cargo features here?) or a normal `alloc::HashMap` depending on std or no_std.

## Case 1: No alloc, no std

## Case 2: Alloc

# Parser/interpreter/executor arch

Targets both PC/rPi with realtime and non-realtime threads, as well as RTIC applications. Use queues/message passing to communicate between components.

- Why is IO in a non-realtime thread?
  - Doesn't really need servo-timing realtime
  - But the limit switches are in the realtime thread because they're handled in the motion controller because it does homing
- How does the HAL work?
  - This is a big one. RTAPI can register functions with `hal_export_funct`. See also <http://www.linuxcnc.org/docs/html/man/man3/hal_create_thread.3hal.html>. Period is decided in `hal_create_thread` call.
- Motion and IO controllers are driven by interpreter
- Motion controller holds and reports distance to go and stuff like that
- Interpreter stores state of variables
  - Updates from HAL come up through the realtime thread and into the interpreter
- How do external realtime plugins work?
  - Dylibs?
  - Expose an API to register multiple RT threads at some tick rate each.
- Expose API to register pins.
  - Might need shared memory for this
  - Test if message passing is fast enough. Might just be with 0MQ

Take a look [here](https://github.com/rust-lang/rust/blob/99e3aef02079e9c10583638520cd0c134dc3a01d/library/std/src/sys/unix/thread.rs) for the Rust thread spawning plumbing. I could copy and paste this into my own Thread struct but with the ability to specify the priority, but using `libc` directly.

The scheduling policy is defined [here](https://wiki.linuxfoundation.org/realtime/documentation/technical_basics/sched_policy_prio/start) (from [this page](https://wiki.linuxfoundation.org/realtime/documentation/howto/applications/application_base)). Tldr - use `SCHED_FIFO`.

So, sleeping on it, I should test message passing instead of shared memory for latency/throughput.

# rPi 64 bit custom RT kernel shit

> NOTE: DOES NOT WORK!

- Env

  - Kernel 5.10
  - Building on 64 bit beta kernel already, downloaded from
  - Using Raspberry Pi 4B

- Deps

  ```bash
  # Probably other stuff too
  sudo apt install bison flex bc
  ```

- Downloaded image from <https://mirrors.edge.kernel.org/pub/linux/kernel/projects/rt/5.10/>
- Did `uname -a`, shows PREEMPT in there but running `latency-test` over SSH shows garbage latency. This is because we don't have the `-rt` suffix on the kernel version.
- See if kernel was compiled with RT: <https://unix.stackexchange.com/questions/424410/enable-config-preempt-rt-full-after-the-kernel-compilation> for var name and <https://github.com/raspberrypi/firmware/issues/442> for instruction:

  ```bash
  sudo modprobe configs
  zcat /proc/config.gz | grep CONFIG_PREEMPT_RT_FULL
  ```

- Because I'm on 5.10.41, I'll check out [this commit](https://github.com/raspberrypi/linux/commit/33069919e2dce440d3b8cd101b18f37bb35bdddf) which is ~600 commits behind current HEAD so I'll use that for `--depth` later.
- Partly follow the rPi kernel guide [here](https://www.raspberrypi.org/documentation/linux/kernel/building.md) but we need to add the preempt-rt patches. These can be applied as per <https://www.raspberrypi.org/documentation/linux/kernel/patching.md>.

  ```bash
  # Clone with a depth large enough to get hold of the commit hash we're interested in
  git clone --depth=600 --branch rpi-5.10.y https://github.com/raspberrypi/linux

  cd linux

  # Check out 5.10.41 commit
  git checkout 33069919e2dce440d3b8cd101b18f37bb35bdddf

  export KERNEL=kernel7l
  # NOTE: This fails on the above commit for some reason. Substitute the actual config instead below
  # make bcm2711_defconfig
  zcat /proc/config.gz > .config
  ```

- Find kernel version with `make kernelversion` and make sure it matches PREEMPT patch version. I guess it could be close but for less error prone build process they should match.
- Apply PREEMPT-RT patch

  ```bash
  wget https://mirrors.edge.kernel.org/pub/linux/kernel/projects/rt/5.10/patch-5.10.41-rt42.patch.gz
  zcat patch-5.10.41-rt42.patch.gz | patch -p1
  ```

- Edit `.config` -> `CONFIG_LOCALVERSION="-v8"` to `CONFIG_LOCALVERSION="-v8-custom-preempt"` just for easy ident.
- As per <https://unix.stackexchange.com/a/653766>, edit `arch/Kconfig` and find `ARCH_SUPPORTS_RT` and change it to:

  ```
  config ARCH_SUPPORTS_RT
    def_bool y
  ```

  This enables the `Fully Preemptible Kernel (Realtime)` option.

- Now to `make menuconfig`

  - Check that `General Setup` -> `Embedded System` is enabled. If not, apparently this can hide the `Fully Preemptible` option
  - `General Setup` -> `Preemption Model` -> set to `Fully Preemptible Kernel (Realtime)`
  - `General setup` -> `Timers subsystem` -> ensure `High Resolution Timer Support` is enabled
  - `Kernel Features` -> `Timer frequency` -> set to `1000 Hz`

- Build kernel

  Note that `zImage` fails. Apparently it's ok to replace with `Image`.

  ```
  make -j4 Image modules dtbs
  sudo make -j4 modules_install
  sudo cp arch/arm/boot/dts/*.dtb /boot/
  sudo cp arch/arm/boot/dts/overlays/*.dtb* /boot/overlays/
  sudo cp arch/arm/boot/dts/overlays/README /boot/overlays/
  sudo cp arch/arm/boot/Image /boot/$KERNEL.img
  ```

Some other links used for the broken instructions above:

- http://web.archive.org/web/20201202023931/https://lemariva.com/blog/2018/02/raspberry-pi-rt-preempt-tutorial-for-kernel-4-14-y
- http://web.archive.org/web/20210205111700/https://lemariva.com/blog/2019/09/raspberry-pi-4b-preempt-rt-kernel-419y-performance-test
- http://web.archive.org/web/20201202024723/https://lemariva.com/blog/2018/07/raspberry-pi-preempt-rt-patching-tutorial-for-kernel-4-14-y
- http://web.archive.org/web/20201130233509/https://lemariva.com/blog/2019/09/raspberry-pi-4b-preempt-rt-kernel-419y-performance-test
- https://mirrors.edge.kernel.org/pub/linux/kernel/projects/rt/5.11/
- https://www.raspberrypi.org/documentation/linux/kernel/building.md
- https://www.raspberrypi.org/documentation/linux/kernel/patching.md
- https://www.raspberrypi.org/forums/viewtopic.php?f=29&t=39951

# rPi Preempt-rt second attempt: 32 bit, install a ready-built DEB package

From here: https://github.com/kdoren/linux/releases/tag/5.10.35-rt39-scarlett-2

- Download all 3 debs
- `sudo apt install ./file.deb` x3
- Before running any of the other commands, do `sudo su`:

  ```bash
  sudo su
  KERN=5.10.35-rt39-v7l+
  mkdir -p /boot/$KERN/overlays/
  cp -d /usr/lib/linux-image-$KERN/overlays/* /boot/$KERN/overlays/
  cp -dr /usr/lib/linux-image-$KERN/* /boot/$KERN/
  touch /boot/$KERN/overlays/README
  mv /boot/vmlinuz-$KERN /boot/$KERN/
  mv /boot/System.map-$KERN /boot/$KERN/
  cp /boot/config-$KERN /boot/$KERN/
  cat >> /boot/config.txt << EOF

  [all]
  kernel=vmlinuz-$KERN
  # initramfs initrd.img-$KERN
  os_prefix=$KERN/
  overlay_prefix=overlays/
  [all]
  EOF
  ```

# Some LinuxCNC-related realtime links

- <https://www.forum.linuxcnc.org/18-computer/36879-raspberry-pi-4-mmap?start=10>
  - Suggests disabling WiFi and using `isolcpus=1,2,3` on <https://www.forum.linuxcnc.org/18-computer/36879-raspberry-pi-4-mmap?start=20>

## Latency comparison

- http://erste.de/raspi4-RT-kernel-howto.html#compile_linCNC_on_Raspi4 - my results look comparable to the screenshots here.
- https://metebalci.com/blog/latency-of-raspberry-pi-4-on-standard-and-real-time-linux-4.19-kernel/ this uses the `cyclictest` util from the preempt-rt patch people which might be helpful for comparing idk.

## rPi LinuxCNC deps install

```bash
sudo apt install  \
debhelper \
libudev-dev \
python-yapps \
tcl8.6-dev \
tk8.6-dev \
libtk-img \
bwidget \
tclx \
libreadline-gplv2-dev \
groff \
python-lxml \
libxmu-dev \
libglu1-mesa-dev \
libgl1-mesa-dev \
libgtk2.0-dev \
libgtk-3-dev \
gettext \
intltool \
autoconf \
libboost-python-dev \
libmodbus-dev \
libusb-1.0-0-dev \
yapps2 \
libtirpc-dev \
imagemagick \
libpng-tools \
libpng12-dev \
libpnglite-dev \
libpnglite0 \
tkpng
```

# `latency-histogram`

Written in TCL

Calls `hal loadrt threads name$i=t_$thd period$i=$::LH($thd,period,ns)`

A function is then added to the thread with `hal addf $::LH($thd,name) t_$thd`

I think the `threads` components comes from `src/hal/components/threads.c`. I think it collects data through pins/params?

`threads` calls `hal_create_thread` in `src/hal/hal_lib.c`.

This calls `rtapi_clock_set_period` (among other things)

The "Unexpected realtime delay" messages comes from `Posix::wait` in `src/rtapi/uspace_rtapi_app.cc`.

I think LCNC realtime threads end up being functions that are called on a period, with the thread sleeping in between?

# LinuxCNC threading model

You spawn a thread with a given period, then use `addf` calls to add a function to the thread. I guess then all the functions are called at the tick rate. If all the functions take too long, what happens next?

# Latency tests for rPi

- Jitter as I've already kinda done
- Histogram GUI
- ZMQ message passing round trip latency or just send latency or whatever
- rPi GPIO toggling
