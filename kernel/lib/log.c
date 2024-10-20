#include "log.h"
#include <status.h>
#include <lib/string.h>
#include <stdint.h>
#include <stdarg.h>

int kprint(const char* str)
{
    while (*str) TRY(kputchar(*str++));
    return 0;
}

int kprintln(const char* str)
{
    kprint(str);
    TRY(kputchar('\n'));
    return 0;
}

static int print_radix(uint64_t number, uint8_t radix, uint8_t min_width)
{
    const char *charmap = "0123456789abcdefghijklmnopqrstuvwxyz";
    if (radix > strlen(charmap)) return -EINVARG;

    uint64_t max = 1;
    // while (min_width-- > 1) max *= radix;
    while (max * radix <= number) max *= radix;
    do {
        TRY(kputchar(charmap[(number / max) % radix]));
        max /= radix;
    } while (max > 0);
    return 0;
}

int kprintf(const char* format, ...)
{
    va_list args;
    va_start(args, format);

    while (*format)
    {
        if (*format == '%')
        {
            if (*++format == '%')
            {
                TRY(kputchar(*format++));
                continue;
            }

            uint8_t size = 0;
            while (isdigit((int)*format))
            {
                size *= 10;
                size += *format++ - '0';
            }

            uint8_t padding = 0;
            if (*format == '.') {
                format++;
                while (isdigit((int)*format))
                {
                    padding *= 10;
                    padding += *format++ - '0';
                }
            }

            if (*format == 'u')
            {
                format++;
                uint64_t arg = UINT64_MAX;
                if (size == 8) arg = va_arg(args, uint32_t) & 0xff;
                else if (size == 16) arg = va_arg(args, uint32_t) & 0xffff;
                else if (size == 32) arg = va_arg(args, uint32_t);
                else if (size == 64) arg = va_arg(args, uint64_t);
                else return -EINVARG;
                TRY(print_radix(arg, 10, padding));
            }
            else if (*format == 'i')
            {
                format++;
                int64_t arg = INT64_MIN;
                if (size == 8) arg = va_arg(args, int32_t);
                else if (size == 16) arg = va_arg(args, int32_t);
                else if (size == 32) arg = va_arg(args, int32_t);
                else if (size == 64) arg = va_arg(args, int64_t);
                else return -EINVARG;
                if (arg < 0)
                {
                    TRY(kputchar('-'));
                    arg = -arg;
                }
                TRY(print_radix(arg, 10, padding));
            }
            else if (*format == 'x')
            {
                format++;
                uint64_t arg = UINT64_MAX;
                if (size == 8) arg = va_arg(args, uint32_t);
                else if (size == 16) arg = va_arg(args, uint32_t);
                else if (size == 32) arg = va_arg(args, uint32_t);
                else if (size == 64) arg = va_arg(args, uint64_t);
                else return -EINVARG;
                TRY(print_radix(arg, 16, padding));
            }
            else if (*format == 'b')
            {
                format++;
                uint64_t arg = UINT64_MAX;
                if (size == 8) arg = va_arg(args, uint32_t);
                else if (size == 16) arg = va_arg(args, uint32_t);
                else if (size == 32) arg = va_arg(args, uint32_t);
                else if (size == 64) arg = va_arg(args, uint64_t);
                else return -EINVARG;
                TRY(print_radix(arg, 2, padding));
            }
            else if (*format == 'p')
            {
                format++;
                if (size != 0 || padding != 0) return -EINVARG;
                TRY(kprint("0x"));
                TRY(print_radix(va_arg(args, size_t), 16, sizeof(size_t) * 2));
            }
            else if (*format == 'c')
            {
                format++;
                if (size != 0 || padding != 0) return -EINVARG;
                TRY(kputchar(va_arg(args, uint32_t)));
            }
            else if (*format == 's')
            {
                format++;
                if (size != 0 || padding != 0) return -EINVARG;
                TRY(kprint(va_arg(args, const char*)));
            }
            else return -EINVARG;
        }
        else TRY(kputchar(*format++));
    }
    va_end(args);
    return 0;
}
