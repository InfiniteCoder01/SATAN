/* === Multiboot Header === */
HEADER_MAGIC    = 0xE85250D6                  # multiboot2 magic number
HEADER_ARCH     = 0                           # arch (0 - i386 protected)
HEADER_LENGTH   = (header_end - header_start) # length of the header
HEADER_CHECKSUM = 0x100000000 - (HEADER_MAGIC + HEADER_ARCH + HEADER_LENGTH) # checksum

.section .multiboot2, "a"
header_start:
    .long HEADER_MAGIC
    .long HEADER_ARCH                                         
    .long HEADER_LENGTH
    .long HEADER_CHECKSUM

    .word 0 # type=0 for end tag
    .word 0 # flags=0
    .long 8 # size=8
header_end:

KERNEL_OFFSET = 0xC0000000

.section .rodata, "a"
# GDT
gdt_start:
gdt_null:
    .long 0x0
    .long 0x0

# offset 0x8
gdt_code:            # CS SHOULD POINT TO THIS
    .word 0xffff     # Segment limit first 0-15 bits
    .word 0          # Base first 0-15 bits
    .byte 0          # Base 16-23 bits
    .byte 0x9a       # Access byte
    .byte 0b11001111 # High 4 bit flags and the low 4 bit flags
    .byte 0          # Base 24-31 bits

# offset 0x10
gdt_data:            # DS, SS, ES, FS, GS
    .word 0xffff     # Segment limit first 0-15 bits
    .word 0          # Base first 0-15 bits
    .byte 0          # Base 16-23 bits
    .byte 0x92       # Access byte
    .byte 0b11001111 # High 4 bit flags and the low 4 bit flags
    .byte 0          # Base 24-31 bits

gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1
    .long gdt_start - KERNEL_OFFSET

CODE_SEG = gdt_code - gdt_start
DATA_SEG = gdt_data - gdt_start

value:
    .word 'b' | 0x100

.section .data, "aw"
.global kernel_page_directory
.align 4096
kernel_page_directory:
    .fill 1024, 4, 0
kernel_page_table_bootstrap:
    .fill 1024, 4, 0
kernel_page_table1_higher_half:
    .fill 1024, 4, 0

.section .stack, "aw"
bootstrap_stack:
    .skip 16384
bootstrap_stack_top:

.section .bootstrap, "ax"
.code32

.type mmap, @function
# esi - start address
# eax - end address
# edi - page table address
# ebx - flags
mmap:
    mov %esi, %ecx
    or %ebx, %ecx
    mov %ecx, (%edi)
    add $4, %edi

    add $4096, %esi
    cmp %eax, %esi
    jb mmap
    ret

.extern kernel_start
.extern data_start
.extern kernel_end
.extern ksetup
.global _start
.type _start, @function
_start:
    cli
    lgdt gdt_descriptor - KERNEL_OFFSET
    mov $DATA_SEG, %cx
    mov %cx, %ds
    mov %cx, %es
    mov %cx, %fs
    mov %cx, %gs
    mov %cx, %ss
    jmp $CODE_SEG, $after_gdt

after_gdt:
    mov $bootstrap_stack_top - KERNEL_OFFSET, %esp
    mov %esp, %ebp
    push %ebx
    push %eax

    # Map first page table (4MB)
    mov $kernel_page_table_bootstrap - KERNEL_OFFSET, %eax
    or $0b00000011, %eax
    mov %eax, kernel_page_directory - KERNEL_OFFSET

    # Map first page table (4MB) in the higher half of the address space
    mov $kernel_page_table1_higher_half - KERNEL_OFFSET, %eax
    or $0b00000011, %eax
    mov %eax, kernel_page_directory - KERNEL_OFFSET + KERNEL_OFFSET / 0x400000 * 4

    mov $0, %esi         # Start address
    mov $0x100000, %eax  # End address
    mov $kernel_page_table_bootstrap - KERNEL_OFFSET, %edi # Page table address
    mov $0b11, %ebx      # Flags
    call mmap
    mov $0x400000, %eax  # End address
    mov $0b01, %ebx      # Flags
    call mmap

    mov $kernel_start - KERNEL_OFFSET, %esi    # Start address
    mov $data_start - KERNEL_OFFSET, %eax      # End address
    # Compute offset into page table
    mov $kernel_start - KERNEL_OFFSET, %edi
    shr $10, %edi
    add $kernel_page_table1_higher_half - KERNEL_OFFSET, %edi # Add to the page table address
    mov $0b11, %ebx                            # Flags
    call mmap

    # Enable paging
    mov $kernel_page_directory - KERNEL_OFFSET, %eax
    mov %eax, %cr3
    mov %cr0, %eax
    or $0x80000000, %eax
    mov %eax, %cr0

    movw value, %ax
    mov %ax, 0xb8000
loop:
    jmp loop
