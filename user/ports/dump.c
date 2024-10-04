#include <stdio.h>

extern char** environ;

int main(int argc, char* argv[]) {
  char** envp = environ;
  printf("%d\n", argc);
  while (*argv) {
    printf("%s", *argv);
    argv++;
    if (*argv == 0) {
      printf("\n");
    } else {
      printf(" ");
    }
  }

  printf("\n");
  for (; *envp; envp++) {
    printf("%s", *envp);
    printf("\n");
  }
}
