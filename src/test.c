// #undef _FORTIFY_SOURCE
#include <stdint.h>

uint8_t testing(uint8_t i) {
  return i + '0';
}
