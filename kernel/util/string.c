#include "kernel/defs.h"

char* safestrcpy(char* dest, const char* src, usize size) {
  if (size == 0) {
    return dest;
  }
  dest[size - 1] = '\0';
  return strncpy(dest, src, size - 1);
}

int strncmp(const char* s1, const char* s2, usize n) {
  while (n-- > 0) {
    u8 u1 = *s1++;
    u8 u2 = *s2++;
    if (u1 != u2) {
      return u1 - u2;
    }
    if (u1 == '\0') {
      return 0;
    }
  }
  return 0;
}

char* strncpy(char* dst, const char* src, usize n) {
  usize i   = 0;
  char* ret = dst;
  while (i++ != n && (*dst++ = *src++))
    ;
  return ret;
}
