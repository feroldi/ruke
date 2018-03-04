# Ruke: experimental microkernel written in Rust

This is a study project aimed to teach me the basics of kernel
innerworkings and development.

## Build

Run `make` in order to correctly compile this project (instead of just
running `cargo`). There's aso no need to create a build directory, the
`Makefile` already handles it.

    make        # Compiles the whole project
    make clean  # Removes the build directory
    make iso    # Generates a bootable iso of the microkernel
    make kernel # Same as `make`
    make run    # Fires up a virtual machine that loads the microkernel

## Project

The kernel can handle allocation of memory pages and output to the
screen. It contains a basic implementation of a frame allocator, page
tables (level 1 to 4), and doesn't provide a way to deallocate pages
heretofore.

Currently, this project is not under development due to loss of attention
to other, more important projects, unfortunately. I might come back to
this some day.

## License

This project is licensed under the MIT License. See LICENSE for more
details.
