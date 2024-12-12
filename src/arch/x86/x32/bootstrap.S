/* === Multiboot Header === */
HEADER_MAGIC    = 0xE85250D6                  /* multiboot2 magic number */
HEADER_ARCH     = 0                           /* arch (0 - i386 protected) */
HEADER_LENGTH   = (header_end - header_start) /* length of the header */
HEADER_CHECKSUM = 0x100000000 - (HEADER_MAGIC + HEADER_ARCH + HEADER_LENGTH) /* checksum */

.section .multiboot2, "a"
header_start:
    .long HEADER_MAGIC
    .long HEADER_ARCH                                         
    .long HEADER_LENGTH
    .long HEADER_CHECKSUM

    .word 0 /* type=0 for end tag */
    .word 0 /* flags=0 */
    .long 8 /* size=8 */
header_end:

.section .bootstrap_stack, "aw", @nobits
stack_bottom:
    .skip 16384 # 16 KiB
stack_top:

.section .bss, "aw", @nobits
    .align 4096
boot_page_directory:
    .skip 4096
boot_page_table1:
    .skip 4096

.section .bootstrap, "ax"
.code32
.extern ksetup
.global _start
.type _start, @function
_start:
    push ebx
    push eax
    call ksetup
