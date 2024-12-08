#include <kernel.h>
#include <status.h>

#include <memory/heap/kheap.h>
#include <memory/paging/paging.h>
#include <idt/idt.h>

#include <lib/log.h>
#include <stddef.h>

int kernel_main()
{
    kprintln("Hello satan!");

    //Initialize the kernel heap
    kheap_init();

    //Initialize the Interrupt Descriptor Table
    idt_init();
    //Enable interrupts
    enable_interrupts();

    // Setup paging
    TRY(paging_init());
    TRY(paging_set(get_current_page_table(), 0x1000, 0x8000, PAGING_FLAG_PRESENT | PAGING_FLAG_WRITEABLE));
    TRY(paging_set(get_current_page_table(), 0x2000, 0x8000, PAGING_FLAG_PRESENT));

    char *testPtr = (char*)0x1000;

    testPtr[0] = 's';
    testPtr[1] = 'a';
    testPtr[2] = 't';
    testPtr[3] = 'a';
    testPtr[4] = 'n';
    testPtr[5] = '\0';

    kprintln((char*)0x2000);

    return SATAN_ALL_OK;
}
