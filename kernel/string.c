#include "types.h"

void* memset(void* dst, int c, uint64 n) {
  char* cdst = (char*)dst;
  uint  i;
  for (i = 0; i < n; i++) {
    cdst[i] = c;
  }
  return dst;
}

int memcmp(const void* v1, const void* v2, uint64 n) {
  const uchar *s1, *s2;

  s1 = v1;
  s2 = v2;
  while (n-- > 0) {
    if (*s1 != *s2) {
      return *s1 - *s2;
    }
    s1++, s2++;
  }

  return 0;
}

void* memmove(void* dst, const void* src, uint64 n) {
  const char* s;
  char*       d;

  if (n == 0) {
    return dst;
  }

  s = src;
  d = dst;
  if (s < d && s + n > d) {
    s += n;
    d += n;
    while (n-- > 0) {
      *--d = *--s;
    }
  } else {
    while (n-- > 0) {
      *d++ = *s++;
    }
  }

  return dst;
}

// memcpy exists to placate GCC.  Use memmove.
void* memcpy(void* dst, const void* src, uint64 n) {
  return memmove(dst, src, n);
}

int strncmp(const char* p, const char* q, uint64 n) {
  while (n > 0 && *p && *p == *q) {
    n--, p++, q++;
  }
  if (n == 0) {
    return 0;
  }
  return (uchar)*p - (uchar)*q;
}

char* strncpy(char* s, const char* t, uint64 n) {
  char* os;

  os = s;
  while (n > 0) {
    --n;
    if ((*s++ = *t++) == 0) {
      break;
    }
  };
  while (n-- > 0) {
    *s++ = 0;
  }
  return os;
}

// Like strncpy but guaranteed to NUL-terminate.
char* safestrcpy(char* s, const char* t, int n) {
  char* os;

  os = s;
  if (n <= 0) {
    return os;
  }
  while (--n > 0 && (*s++ = *t++) != 0)
    ;
  *s = 0;
  return os;
}

uint64 strlen(const char* s) {
  int n;

  for (n = 0; s[n]; n++)
    ;
  return n;
}
