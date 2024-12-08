Note: Suggestions for naming things are welcome

## Terminology
- Let's call a single process an object (No particular reason for the name object, other suggestions are welcome).
- Now, object can have multiple threads, just like processes in other kernels.
- Unlike processes in other kernels, let's allow object to have 0 threads
(it won't be processing anything and CPU scheduler will ignore it. That's why I don't want to call it a process).

Example object with 0 threads:
```C
#include <kernel/object.h>

int main ()
{
    automaticTerminate(false); // By default, once thread count goes to zero, object would get terminated automatically. Now you would have to call `exit(0);` on it to terminate it.
    return 0;
}
```

Now, an object like this would probably get terminated by the kernel,
because it isn't very useful. What could be useful is something like this:
```C
#include <kernel/object.h>
#include <stdio.h>

//              V RPC gives a handle to an object that called the method
void sayHi(object_handle _caller)
{
    puts("Hello, World!");
}

int main ()
{
    automaticTerminate(false);
    registerRPC("sayHi", sayHi, 0, 0); // Here is why we need objects with 0 threads. RPCs (Remote Procedure Calls)
    // make them almost like libraries with their own address space. Now, ideally we would make this code an actual
    // library and map it to all objects that want to use it, but make it protected and only accessible with an
    // interface such as system calls. But we can't do that without putting it into the kernel directly, which is
    // against microkernel design.
    return 0;
}
```

## Creating an object
To create an object (like process in other OSes), a function/syscall similar to this could be used:
```C
object_handle createObject(void *data, size_t size, size_t entry_point);
```
Where `object_handle` is probably an index into list of object descriptors in kernel and `data` has to be
page-aligned (because we will need to memory map it).

Under the hood, it would probably:
- Create some sort of `ObjectDescriptor` struct and add it to the list
- Create an address space associated with this object descriptor
- Memory map data and size to this address space
- Start execution from the `entry_point` (offset into `data`).

Filesystem API would most likely provide a function to simplify
loading executables from files (file format is open for discussion):
```C
object_handle loadObject(path_t path);
```
It would also take into consideration permissions of the object to
read and exectute files at this path.

Under the hood, it would probably:
- Memory map the file into RAM
- Call `createObject` from kernel

## Permissions
Allowing any object to nuke the whole system without asking the user is not the best approach.
Thus, we need a permission system, which would limit what objects can do.

Assuming we are going for microkernel approach, there shouldn't be many basic permissions from objects
(by basic I mean permissions granted by the kernel). Thus, permissions are mostly managed by other
components (filesystem driver, for example, controls which objects can access which files).

I don't yet have an idea how to implement a permission system

## Modularity
Assuming we are using microkernel approach.
Kerenl manages:
- memory (create and free page tables, mmap a page to a page table, create shared memory region, etc. There can only be one page table per object)
- scheduling
- IPC (Inter-process communication), such as RPC (Remote procedure calls), shared memory, something else

Here is a possible boot process:
- GRUB (or other multiboot2 bootloader) loads kernel and modules (modules being file system driver and init system, for example)
- kernel sets up page table for itself, mapping all kernel code to the upper part of the address space.
- After we enable paging, kernel sets up IDT, heap, stack, creates objects for all modules loaded by the bootloader.
- Init system launches shell from the filesystem.

Kernel should provide following ways to communicate between objects:
- Shared memory. Function/syscall `void *mmap(size_t size);` would mmap more memory into this object, size is going to be rounded up to be PAGE_SIZE multiple.
Function/syscall `ssize_t share(object_handle object, void *region, size_t size);` would share `region` with `object` and return address in `object`'s address space,
if object allows sharing memory with it
- RPC, not sure about the API. Something like this could be integrated with protobuf and gRPC data structures.
Function/syscall `int registerRPC(const char *name, void *function, size_t params_size)` would register an RPC function.
Function/syscall `rpc_func_handle getRPC(object_handle object, const char *name)` would return RPC function handle (probably just an index).
Function/syscall `callRPC`, and I have no idea of a signature for it, which would call the RPC function and return the value

### Filesystem
Filesystem object could provide the following RPC methods:
- `void *mmapFile(path_t path);` creates a shared memory region, which represents the open file and is updated.
- `int closeFile(void *file);` closes previously mapped file
- Something to list directory
- Something to create/remove files
- Something to manage permissions
