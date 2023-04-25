# glOS2 - Second time's the charm

Lookie here, another OS from the mind of. Let's just hope that this isn't as much of a shitshow now that I'm actually trying to do things properly.

## Legaliese

I'm NOT going to be rolling my own bootloader this time. We will be using [Multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) with a default bootmgr of [GRUB2](https://www.gnu.org/software/grub/index.html). Of course this isn't directly packaged with the OS and will have to be installed seperately for development purposes. However, I am planning to take an ArchLinux style approach to installing this shit, so GRUB2 is likely to be packaged with any release of this OS. Pretty much this means the GPL license from GRUB2 is going to have to be included in here, so I'll place it [here](LICENSES/GPL-3_0).

As for the License of glOS2, It is [`LGPL-2.1-only`](https://spdx.org/licenses/LGPL-2.1-only.html). This is so any parts of glOS that CAN be used as a library, such as any of the syscalls, are able to be used freely in propietary software. However any modifications to the source of glOS MUST be kept OpenSource & redistributed under the terms of `LGPL-2.1-only`. Cause if you fix something, submit a PR so everyone else can easily enjoy it.

## Here's the plan

Now with that legal disclosure out of the way, what is the actual plan this time?

- [x] Barebones Kernel
- [ ] Text Console
  - [x] GDT
  - [ ] IDT
  - [ ] Screen Scrolling
- [ ] Heap Memory Allocation
- [ ] Framebuffer via VESA
  - [ ] Fonts
- [ ] Windows
  - [ ] Rectangles
  - [ ] Viewports
  - [ ] Multiple Windows
  - [ ] Window Manager (Tiled?)
- [ ] UI Library
- [ ] Filesystem
- [ ] Programs (ELF)
- [ ] 64-bit Support (oh god dual-wielding)
- [ ] Full Rust Support for Programs (`std`, `alloc`, etc)
  - [ ] libstdc implementation (which also means C/C++ support)
  - [ ] Compiler Toolchain
- [ ] Security
  - [ ] Seperate User-space & Kernel-space
  - [ ] Permissions System
  - [ ] Users
- [ ] Mouse Support
- [ ] Audio
- [ ] Packages
  - [ ] Package Management
- [ ] Installation Media/Guide
- [ ] Multithreading
- [ ] Networking

From here I have no idea what I'd do. I'm not even sure if I'll get to the end of this list, but holy shit if I do it'll be my greatest achievement.

## How can I use this?

It's actually really simple suprisingly. Just install [Rust](https://www.rust-lang.org/), [cargo-make](https://sagiegurari.github.io/cargo-make/), [QEMU](https://www.qemu.org/), [GRUB2](https://www.gnu.org/software/grub/index.html) and you're set!

```sh
# First, make sure you have the repo cloned
git clone https://github.com/GamingLiamStudios/glOS2
cd glOS2

# Then make sure you have the correct dependencies
makers install-deps

# Then just
makers build-rls # generates glos.iso
makers run # launches qemu w/ logs
```

If you're going to be updating `link.ld` - make sure you run `makers clean` after, otherwise your changes won't propagate. (Annoying, I know.)
