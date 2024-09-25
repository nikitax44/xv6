#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main(int argc, char* argv[]) {
  int i;

  for (i = 1; i < argc; i++) {
    const char* val = getenv(argv[i]);
    if (val) {
      printf("set %s=%s\n", argv[i], val);
    } else {
      printf("unset %s\n", argv[i]);
    }
  }
}
