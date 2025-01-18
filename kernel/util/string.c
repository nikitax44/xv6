#include "kernel/defs.h"

char* safestrcpy(char* dest, const char* src, usize size) {
  if (size == 0) {
    return dest;
  }
  dest[size - 1] = '\0';
  return strncpy(dest, src, size - 1);
}
