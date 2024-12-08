#ifndef PAGING_H
#define PAGING_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "arch/paging/paging.h"

enum
{
    PAGING_FLAG_PRESENT = 0b00000001,
    PAGING_FLAG_WRITEABLE = 0b00000010,
    PAGING_FLAG_EXECUTABLE = 0b00000100,
    PAGING_FLAG_PROTECTED = 0b00001000,
    PAGING_FLAG_DISABLE_CACHE = 0b00010000, // Might not be implemented in all systems
    PAGING_FLAGS_ALL = 0b00011111,
};

int paging_init();
page_table_handle get_kernel_page_table();

page_table_handle get_current_page_table();
page_table_handle paging_new_table();
int paging_switch(page_table_handle table);
int paging_free_table(page_table_handle table);

int paging_set(page_table_handle page_table, size_t virtaul_address, size_t physical_address, uint8_t flags);
bool paging_is_aligned(size_t addr);

#endif
