#include "paging_impl.h"
#include <memory/heap/kheap.h>
#include <memory/paging/paging.h>
#include <status.h>

#define PAGING_TOTAL_ENTRIES_PER_TABLE 1024

#define PAGING_IS_PRESENT 0b00000001
#define PAGING_IS_WRITEABLE 0b00000010
#define PAGING_ACCESS_FROM_ALL 0b00000100
#define PAGING_WRITE_THROUGH 0b00001000
#define PAGING_CACHE_DISABLED 0b00010000

static page_table_handle kernel_page_table = 0;

static int convert_flags(uint8_t *flags)
{
    uint8_t input_flags = *flags;
    if (input_flags & ~PAGING_FLAGS_ALL) return -EINVARG;

    *flags = 0;
    if (input_flags & PAGING_FLAG_PRESENT) *flags |= PAGING_IS_PRESENT;
    if (input_flags & PAGING_FLAG_WRITEABLE) *flags |= PAGING_IS_WRITEABLE;
    // if (input_flags & PAGING_FLAG_EXECUTABLE) *flags |= PAGING_IS_EXECUTABLE;
    if (!(input_flags & PAGING_FLAG_PROTECTED)) *flags |= PAGING_ACCESS_FROM_ALL;
    if (input_flags & PAGING_FLAG_DISABLE_CACHE) *flags |= PAGING_CACHE_DISABLED;
    return 0;
}

int paging_init()
{
    kernel_page_table = paging_new_table();
    paging_switch(kernel_page_table);
    ENABLE_PAGING();
    return SATAN_ALL_OK;
}

page_table_handle get_kernel_page_table()
{
    return kernel_page_table;
}

page_table_handle get_current_page_table()
{
    register page_table_handle dst;
    __asm__ volatile(
        "mov %%cr3, %[dst]"
        : [dst] "=rm"(dst));
    return dst;
}

page_table_handle paging_new_table()
{
    size_t *table = kmalloc(sizeof(size_t) * PAGING_TOTAL_ENTRIES_PER_TABLE);
    int offset = 0;
    for (int i = 0; i < PAGING_TOTAL_ENTRIES_PER_TABLE; i++)
    {
        size_t *entry = kmalloc(sizeof(size_t) * PAGING_TOTAL_ENTRIES_PER_TABLE);
        for (int b = 0; b < PAGING_TOTAL_ENTRIES_PER_TABLE; b++)
        {
            entry[b] = (offset + (b * PAGE_SIZE)) | PAGING_IS_PRESENT | PAGING_IS_WRITEABLE;
        }
        offset += (PAGING_TOTAL_ENTRIES_PER_TABLE * PAGE_SIZE);
        table[i] = (size_t)entry | PAGING_IS_PRESENT | PAGING_IS_WRITEABLE;
    }

    return (size_t)table;
}

int paging_switch(page_table_handle table)
{
    __asm__ volatile("mov %0, %%cr3" : "=rm"(table));
    return SATAN_ALL_OK;
}

// int paging_free_table(page_table_handle table);

static void compute_addresses(page_table_handle page_table, size_t virtual_address, size_t *table_address, size_t *table_offset)
{
    size_t directory_index = ((size_t)virtual_address / (PAGING_TOTAL_ENTRIES_PER_TABLE * PAGE_SIZE));
    size_t table_index = ((size_t)virtual_address % (PAGING_TOTAL_ENTRIES_PER_TABLE * PAGE_SIZE) / PAGE_SIZE);
    *table_address = page_table + directory_index * sizeof(size_t);
    *table_offset = table_index * sizeof(size_t);
}

int paging_set(page_table_handle page_table, size_t virtual_address, size_t physical_address, uint8_t flags)
{
    if (!paging_is_aligned(virtual_address)) return -EINVARG;
    if (!paging_is_aligned(physical_address)) return -EINVARG;
    TRY(convert_flags(&flags));
    size_t table_address, table_offset;
    compute_addresses(page_table, virtual_address, &table_address, &table_offset);
    size_t value = physical_address | flags;

    DISABLE_PAGING();
    __asm__ volatile(
        "mov (%[table_address]), %%eax\n\t"
        "and $0xfffff000, %%eax\n\t"
        "mov %[value], (%%eax, %[table_offset])\n\t"
        :
        : [table_address] "ir"(table_address),
          [table_offset] "ir"(table_offset),
          [value] "ir"(value)
        : "eax", "edx", "memory");
    ENABLE_PAGING();
    return 0;
}

bool paging_is_aligned(size_t addr)
{
    return (addr % PAGE_SIZE) == 0;
}
