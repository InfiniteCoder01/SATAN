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

## Help!!!
Here are some things you could help with:
- Current paging/address space implementation dealt me so much pain that I can't even look at it now.
Would be nice if anyone could help rewrite it in a rusty way!
- Find a way to make freeing in ZonedBuddy page allocator checked (you can free a random page rigth now and it won't even panic)
- Find a way to also check freeing in NestedPageTable
