#include <stdio.h>
#include <unistd.h>

// extern char* environ[];

int main(int argc, char* argv[], char* envp[]) {
  int i;

  for (i = 1; i < argc; i++) {
    printf("%s", argv[i]);
    if (i + 1 < argc) {
      printf(" ");
    } else {
      printf("\n");
    }
  }
  for (; *envp; envp++) {
    printf("%s", *envp);
    printf("\n");
  }
}
