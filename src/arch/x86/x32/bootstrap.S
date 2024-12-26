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

gdt_tss:             # TS
    .word 0xffff     # Segment limit first 0-15 bits
    .word 0          # Base first 0-15 bits
    .byte 0          # Base 16-23 bits
    .byte 0x89       # Access byte
    .byte 0          # High 4 bit flags and the low 4 bit flags
    .byte 0          # Base 24-31 bits

gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1
    .long gdt_start - KERNEL_OFFSET

CODE_SEG = gdt_code - gdt_start
DATA_SEG = gdt_data - gdt_start
TSS_SEG = gdt_tss - gdt_start

.section .data, "aw"
.align 4096
.global kernel_page_directory
kernel_page_directory:
    .fill 1024, 4, 0
kernel_page_table_bootstrap:
    .fill 1024, 4, 0
kernel_page_table1_higher_half:
    .fill 1024, 4, 0

.global kernel_tmp_page_entry_address
kernel_tmp_page_entry_address:
    .long 0

.section .stack, "aw"
bootstrap_stack:
    .skip 16384
bootstrap_stack_top:
tss_stack:
    .skip 16384
tss_stack_top:

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

    # Setup TSS
    mov tss_stack_top - tss_stack - 1, %cx
    mov %cx, gdt_tss

    mov tss_stack, %ecx
    mov %cx, gdt_tss + 2
    shr $16, %ecx
    mov %cl, gdt_tss + 4
    shr $8, %ecx
    mov %cl, gdt_tss + 7

    # Setup GDT
    lgdt gdt_descriptor - KERNEL_OFFSET
    mov $DATA_SEG, %cx
    mov %cx, %ds
    mov %cx, %es
    mov %cx, %fs
    mov %cx, %gs
    mov %cx, %ss
    mov $TSS_SEG, %cx
    ltr %cx
    jmp $CODE_SEG, $after_gdt

after_gdt:
    mov $bootstrap_stack_top - KERNEL_OFFSET, %esp
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

    # Check if kernel fits
    mov $KERNEL_OFFSET + 0x400000, %eax
    cmp $kernel_end, %eax
    jb kernelTooBig
    
    mov $kernel_start - KERNEL_OFFSET, %esi    # Start address
    mov $data_start - KERNEL_OFFSET, %eax      # End address
    # Compute offset into page table
    mov $kernel_start - KERNEL_OFFSET, %edi
    shr $10, %edi
    add $kernel_page_table1_higher_half - KERNEL_OFFSET, %edi # Add to the page table address
    mov $0b01, %ebx                            # Flags
    call mmap
    mov $kernel_end - KERNEL_OFFSET, %eax      # End address
    mov $0b11, %ebx                            # Flags
    call mmap

    # Setup tmp page
    sub $4, %edi
    add $KERNEL_OFFSET, %edi
    mov %edi, kernel_tmp_page_entry_address - KERNEL_OFFSET

    # Enable paging
    mov $kernel_page_directory - KERNEL_OFFSET, %eax
    mov %eax, %cr3
    mov %cr0, %eax
    or $0x80000000, %eax
    mov %eax, %cr0

    # Fix stack pointer
    add $KERNEL_OFFSET, %esp

    # Note that multiboot2 args are on top of the stack
    call ksetup
    cli
hlt.loop:
    hlt
    jmp hlt.loop

kernelTooBig:
    movw $'K' | 0x0400, 0xb8000
    movw $'e' | 0x0400, 0xb8002
    movw $'r' | 0x0400, 0xb8004
    movw $'n' | 0x0400, 0xb8006
    movw $'e' | 0x0400, 0xb8008
    movw $'l' | 0x0400, 0xb800a
    movw $' ' | 0x0400, 0xb800c
    movw $'t' | 0x0400, 0xb800e
    movw $'o' | 0x0400, 0xb8010
    movw $'o' | 0x0400, 0xb8012
    movw $' ' | 0x0400, 0xb8014
    movw $'b' | 0x0400, 0xb8016
    movw $'i' | 0x0400, 0xb8018
    movw $'g' | 0x0400, 0xb801a
    movw $'!' | 0x0400, 0xb801c
kernelTooBig.loop:
    jmp kernelTooBig.loop
