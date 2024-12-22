#pragma once
#include "hardware/riscv.h"
#include "kernel/errno.h"
#include "types.h"

struct buf;
struct context;
struct file;
struct proc;
struct spinlock;
struct sleeplock;
struct stat;
typedef enum { SEEK_SET = 0, SEEK_CUR = 1, SEEK_END = 2 } WHENCE;

// console.c
void consoleinit(void);
void consoleintr(int);
void consputc(int);

// exec.c
// gcc does not recognize that they're the same
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wbuiltin-declaration-mismatch"
int execve(str, str*, str*);
#pragma GCC diagnostic pop

// proc.c
u32          cpuid(void);
void         exit(int);
int          fork(void);
int          growproc(int);
pagetable_t  proc_pagetable(struct proc*);
void         proc_freepagetable(pagetable_t, u64);
int          kill(int);
void         kill_all(void);
int          killed(struct proc*);
void         setkilled(struct proc*);
struct cpu*  mycpu(void);
struct proc* myproc(void);
void         procinit(void);
void         scheduler(void) __attribute__((noreturn));
void         sched(void);
void         sleep(void*, struct spinlock*);
void         userinit(void);
int          wait(u64);
void         wakeup(void*);
void         yield(void);
int          either_copyout(int user_dst, u64 dst, void* src, u64 len);
int          either_copyin(void* dst, int user_src, u64 src, u64 len);
void         procdump(void);

// swtch.S
void swtch(struct context*, struct context*);

// spinlock.c
void acquire(struct spinlock*);
int  holding(struct spinlock*);
void initlock(struct spinlock*, char*);
void release(struct spinlock*);
void push_off(void);
void pop_off(void);

// sleeplock.c
void acquiresleep(struct sleeplock*);
void releasesleep(struct sleeplock*);
int  holdingsleep(struct sleeplock*);
void initsleeplock(struct sleeplock*, char*);

// string.c
char* safestrcpy(char*, const char*, usize);

// syscall.c
void argint(int, int*);
int  argstr(int, char*, int);
void argaddr(int, u64*);
int  fetchstr(u64, char*, int);
int  fetchaddr(u64, u64*);
void syscall(void);

// trap.c
extern u32             ticks;
void                   trapinit(void);
void                   trapinithart(void);
void                   timerinithart(void);
extern struct spinlock tickslock;
void                   usertrapret(void);

// uart.c
void uartinit(void);
void uartintr(void);
void uartputc(char);
void uartputc_sync(char);
int  uartgetc(void);

// vm.c
void        kvminithart(void);
pagetable_t uvmcreate(void);
void        uvmfirst(pagetable_t, u8*, u32);
u64         uvmalloc(pagetable_t, u64, u64, int);
u64         uvmdealloc(pagetable_t, u64, u64);
int         uvmcopy(pagetable_t, pagetable_t, u64);
void        uvmfree(pagetable_t, u64);
void        uvmunmap(pagetable_t, u64, u64, int);
void        uvmclear(pagetable_t, u64);
pte_t*      walk(pagetable_t, u64, int);
int         copyout(pagetable_t, u64, const u8*, u64);
int         copyin(pagetable_t, char*, u64, u64);
int         copyinstr(pagetable_t, char*, u64, u64);

// plic.c
void plicinit(void);
void plicinithart(void);
int  plic_claim(void);
void plic_complete(int);

// #### Rust ####
void dumpconf(void);
int  printf(const char*, ...) __attribute__((format(printf, 1, 2)));
void shutdown(void) __attribute__((noreturn));
void reboot(void) __attribute__((noreturn));
// panic
void panic(char*) __attribute__((noreturn));
void testpanic(void) __attribute__((noreturn));
// fs
errno_t rs_fs_create(const char* path);
errno_t rs_fs_mkdir(const char* path);
errno_t rs_fs_mknod(const char* path, u32 major, u32 minor);
errno_t rs_fs_unlink(const char* path);
errno_t rs_fs_hard_link(const char* old, const char* new);
errno_t rs_fs_symbolic_link(const char* old, const char* new);

// fs::pipe
// returns pair
errno_t rs_pipe_alloc(struct file** rp, struct file** wp);

// fs::file
struct file* rs_file_open(const char* path);
void         rs_file_close(struct file* file);
struct file* rs_file_dup(struct file* file);
struct stat  rs_file_stat(struct file* file);
usize        rs_file_read(struct file* file, u8* buf, usize len);
usize        rs_file_write(struct file* file, const u8* buf, usize len);
isize        rs_file_seek(struct file* file, i64 offset, WHENCE whence);

// vm
void kvminit(void);
int  mappages(pagetable_t, u64, u64, u64, int);
u64  walkaddr(pagetable_t, u64);

// disk
void rs_disk_intr(void);

// kalloc
extern void* kalloc(void);
extern void  kfree(void*);
extern void  kinit(void);
extern u64   free_pages(void);

// compiler builtins
usize strlen(const char*);
void  memset(void*, u8, usize);
void* memmove(void*, const void*, usize);
char* strcpy(char*, const char*);
char* strncpy(char*, const char*, usize);

// number of elements in fixed-size array
#define NELEM(x) (sizeof(x) / sizeof((x)[0]))
#define TODO     panic("todo");
