#pragma once
#include "hardware/riscv.h"
#include "types.h"

struct list;
struct buf;
struct context;
struct file;
struct inode;
struct pipe;
struct proc;
struct spinlock;
struct sleeplock;
struct stat;
struct superblock;
typedef enum { SEEK_SET, SEEK_CUR, SEEK_END } WHENCE;

// bio.c
void        binit(void);
struct buf* bread(u32, u32);
void        brelse(struct buf*);
void        bwrite(struct buf*);
void        bpin(struct buf*);
void        bunpin(struct buf*);

// console.c
void consoleinit(void);
void consoleintr(int);
void consputc(int);

// list.c
void  lst_init(struct list*);
void  lst_remove(struct list*);
void  lst_push(struct list*, void*);
void* lst_pop(struct list*);
void  lst_print(struct list*);
int   lst_empty(struct list*);
void  lst_extend_move(struct list*, struct list*);

// exec.c
// gcc does not recognize that they're the same
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wbuiltin-declaration-mismatch"
int execve(str, str*, str*);
#pragma GCC diagnostic pop

// file.c
struct file* filealloc(void);
void         fileclose(struct file*);
struct file* filedup(struct file*);
void         fileinit(void);
int          fileread(struct file*, u64, int n);
int          filestat(struct file*, u64 addr);
int          fileseek(struct file*, u64, WHENCE n);
int          filewrite(struct file*, u64, int n);

// fs.c
void          fsinit(int);
int           dirlink(struct inode*, char*, u32);
struct inode* dirlookup(struct inode*, char*, u32*);
struct inode* ialloc(u32, short);
struct inode* idup(struct inode*);
void          iinit(void);
void          ilock(struct inode*);
void          iput(struct inode*);
void          iunlock(struct inode*);
void          iunlockput(struct inode*);
void          iupdate(struct inode*);
int           namecmp(str, str);
struct inode* namei(str);
struct inode* nameiparent(char*, char*);
u32           readi(struct inode*, int, u64, u32, u32);
void          stati(struct inode*, struct stat*);
u32           writei(struct inode*, int, u64, u32, u32);
void          itrunc(struct inode*);

// log.c
void initlog(int, struct superblock*);
void log_write(struct buf*);
void begin_op(void);
void end_op(void);

// pipe.c
int  pipealloc(struct file**, struct file**);
void pipeclose(struct pipe*, int);
int  piperead(struct pipe*, u64, int);
int  pipewrite(struct pipe*, u64, int);

// proc.c
u32          cpuid(void);
void         exit(int);
int          fork(void);
int          growproc(int);
void         proc_mapstacks(pagetable_t);
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
void        kvminit(void);
void        kvminithart(void);
int         mappages(pagetable_t, u64, u64, u64, int);
pagetable_t uvmcreate(void);
void        uvmfirst(pagetable_t, u8*, u32);
u64         uvmalloc(pagetable_t, u64, u64, int);
u64         uvmdealloc(pagetable_t, u64, u64);
int         uvmcopy(pagetable_t, pagetable_t, u64);
void        uvmfree(pagetable_t, u64);
void        uvmunmap(pagetable_t, u64, u64, int);
void        uvmclear(pagetable_t, u64);
pte_t*      walk(pagetable_t, u64, int);
u64         walkaddr(pagetable_t, u64);
int         copyout(pagetable_t, u64, const u8*, u64);
int         copyin(pagetable_t, char*, u64, u64);
int         copyinstr(pagetable_t, char*, u64, u64);
int         map_stack(usize);
void        unmap_stack(usize);

// plic.c
void plicinit(void);
void plicinithart(void);
int  plic_claim(void);
void plic_complete(int);

// virtio_disk.c
void virtio_disk_init(void);
void virtio_disk_rw(struct buf*, int);
void virtio_disk_intr(void);

// #### Rust ####
extern void dumpconf(void);
extern int  printf(const char*, ...) __attribute__((format(printf, 1, 2)));
extern void shutdown(void) __attribute__((noreturn));
extern void reboot(void) __attribute__((noreturn));
// panic
extern void panic(char*) __attribute__((noreturn));
extern void testpanic(void) __attribute__((noreturn));

// kalloc
extern void* kalloc(void);
extern void* alloc(u64);
extern void* free(u64, u64);
extern void  kfree(void*);
extern void  kinit(void);
extern u64   free_pages(void);

// compiler builtins
usize strlen(const char*);
void  memset(void*, u8, usize);
void* memmove(void*, const void*, usize);
char* strcpy(char*, const char*);
int   strncmp(const char*, const char*, usize);
char* strncpy(char*, const char*, usize);

// number of elements in fixed-size array
#define NELEM(x) (sizeof(x) / sizeof((x)[0]))
#define TODO     panic("todo");
