#include "types.h"

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
