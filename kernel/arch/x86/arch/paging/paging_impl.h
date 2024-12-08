#ifndef ARCH_PAGING_PAGING_IMPL_H
#define ARCH_PAGING_PAGING_IMPL_H

#include <stddef.h>

#define ENABLE_PAGING() __asm__ volatile( \
    "mov %%cr0, %%eax\n\t"                \
    "or $0x80000000, %%eax\n\t"           \
    "mov %%eax, %%cr0" ::: "eax")

#define DISABLE_PAGING() __asm__ volatile( \
    "mov %%cr0, %%eax\n\t"                 \
    "and $~0x80000000, %%eax\n\t"          \
    "mov %%eax, %%cr0" ::: "eax")

struct PageInfo
{
    size_t uses;
};

extern size_t page_info_table_address;
extern size_t total_page_count;

#endif
