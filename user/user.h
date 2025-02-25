#include "kernel/types.h"
#include <string.h>
#define stderr 2

struct stat;

// system calls
int   fork(void);
void  exit(int) __attribute__((noreturn));
int   wait(int*);
int   pipe(int*);
int   write(int, const void*, int);
int   read(int, void*, int);
isize seek(int, isize, int);
int   close(int);
int   kill(int);
int   exec(str, char**);
int   execve(str, char* const*, char* const*);
int   open(str, int);
int   mknod(str, short, short);
int   unlink(str);
int   fstat(int fd, struct stat*);
int   link(str, str);
int   mkdir(str);
int   chdir(str);
int   dup(int);
int   getpid(void);
char* sbrk(int);
int   sleep(int);
int   uptime(void);
int   shutdown(void);

// ulib.c
int   stat(str, struct stat*);
void  fdprintf(int, str, ...) __attribute__((format(printf, 2, 3)));
void  printf(str, ...) __attribute__((format(printf, 1, 2)));
char* gets(char*, int max);
int   atoi(str);
