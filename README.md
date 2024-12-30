# SATAN - an opensource kernel
## Building
If you want to build the kernel, you will need to setup a cross-compiler as described [here](https://wiki.osdev.org/GCC_Cross-Compiler) and export `PREFIX` environment variable.
Then, run the build script by issuing `./build.sh`

## Compatibility
|    Arch    | Compatibility | Implementation notes |
|------------|---------------|----------------------|
| x86 (i486) |     Works     |                      |
| x86 (i386) |  Unsupported  |     TLB flushing     |
|   x86_64   |     TODO      |                      |
