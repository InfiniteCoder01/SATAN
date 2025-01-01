ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

KERNEL_OFFSET = 0xC0000000;

SECTIONS {
	. = 1M;

	.boot : {
		KEEP(*(.multiboot2))
		*(.bootstrap)
	}

	. += KERNEL_OFFSET;
	
	/* Read-only code */
	.text ALIGN (4K) : AT (ADDR (.text) - KERNEL_OFFSET) {
		/* Add a symbol that indicates the start address of the kernel. */
		kernel_start = .;
		*(.text .text.*)
	}

	/* Read-only data, page aligned to allow use of the no-execute feature */
	.rodata ALIGN (4K) : AT (ADDR (.rodata) - KERNEL_OFFSET) {
		*(.rodata .rodata.*)
	}
		
	/* Read-write data, page aligned for the .padata section */
	.data ALIGN (4K) : AT (ADDR (.data) - KERNEL_OFFSET) {
		data_start = .;
		*(.padata)
		*(.data .data.*)
		*(.bss .bss.*)
		*(.stack)
	}

	/* Add a symbol that indicates the end address of the kernel. Add 4KiB for a TMP_PAGE */
	kernel_end = .;

	kernel_tmp_page_address = ALIGN(4K);

	/* Add a symbol that indicates the end address of the space reserved for kernel. */
	kernel_reserved_end = kernel_tmp_page_address + 4096;

}
