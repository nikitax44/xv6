#pragma once
// Physical memory layout

// qemu -machine virt is set up like this,
// based on qemu's hw/riscv/virt.c:
//
// 00001000 -- boot ROM, provided by qemu
// 02000000 -- CLINT
// 0C000000 -- PLIC
// 10000000 -- uart0
// 10001000 -- virtio disk
// 80000000 -- boot ROM jumps here in machine mode
//             -kernel loads the kernel here
// unused RAM after 80000000.

// the kernel uses physical memory thus:
// 80000000 -- entry.S, then kernel text and data
// end -- start of kernel page allocation area
// PHYSTOP -- end RAM used by the kernel

#define TEST0          0x100000L
#define TEST0_SHUTDOWN 0x00005555
#define TEST0_REBOOT   0x00007777

#define FW_CFG           0x10100000L
#define FW_CFG_SEL       (FW_CFG + 0x08)
#define FW_CFG_DAT       (FW_CFG + 0x00)
#define FW_CFG_DMA       (FW_CFG + 0x10)
#define FW_CFG_MAGIC     0x554d4551         // "QEMU"
#define FW_CFG_DMA_MAGIC 0x47464320554d4551 // "QEMU CFG"
#define FW_CFG_SIGNATURE 0x0000
#define FW_CFG_ID        0x0001
#define FW_CFG_FILE_DIR  0x0019

// qemu puts UART registers here in physical memory.
#define UART0     0x10000000L
#define UART0_IRQ 10

// virtio mmio interface
#define VIRTIO0     0x10001000
#define VIRTIO0_IRQ 1

// qemu puts platform-level interrupt controller (PLIC) here.
#define PLIC                 0x0c000000L
#define PLIC_PRIORITY        (PLIC + 0x0)
#define PLIC_PENDING         (PLIC + 0x1000)
#define PLIC_SENABLE(hart)   (PLIC + 0x2080 + (hart) * 0x100)
#define PLIC_SPRIORITY(hart) (PLIC + 0x201000 + (hart) * 0x2000)
#define PLIC_SCLAIM(hart)    (PLIC + 0x201004 + (hart) * 0x2000)

// map the trampoline page to the highest address,
// in both user and kernel space.
#define TRAMPOLINE (MAXVA - PGSIZE)

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
#define KSTACK(p) (TRAMPOLINE - ((p) + 1) * 3 * PGSIZE)

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)
#define TRAPFRAME (TRAMPOLINE - PGSIZE)
