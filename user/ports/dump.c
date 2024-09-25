#include <stdio.h>
#include <unistd.h>

int main(int argc, char* argv[], char* envp[]) {
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
