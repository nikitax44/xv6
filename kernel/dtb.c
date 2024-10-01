#include "dtb.h"

struct FWCfgFiles* fw_dir;

static inline u16 bswap16(u16 x) { return __builtin_bswap16(x); }
static inline u32 bswap32(u32 x) {
  return ((u32)bswap16(x & 0xffff) << 16) | bswap16(x >> 16);
}
static inline u64 bswap64(u64 x) {
  return ((u64)bswap32(x & 0xffffffff) << 32) | bswap32(x >> 32);
}
#define FW_GET_DAT(bits) (bswap##bits(*(volatile u##bits*)FW_CFG_DAT))
#define FW_GET_DMA(bits) (bswap##bits(*(volatile u##bits*)FW_CFG_DMA))

static void fw_select(u16 key) {
  *(volatile u16*)FW_CFG_SEL = __builtin_bswap16(key);
}

static u64 fw_dat_raw() { return *(volatile u64*)FW_CFG_DAT; }
static u64 fw_dma_raw() { return *(volatile u64*)FW_CFG_DMA; }

static void fw_verify(void) {
  fw_select(0);
  if (fw_dat_raw() != FW_CFG_MAGIC) {
    panic("QEMU FW_CFG magic");
  }
  fw_select(FW_CFG_ID);
  if ((fw_dat_raw() & 0x3) == 0) {
    panic("QEMU FW_CFG features");
  }
  fw_select(FW_CFG_SIGNATURE);
  if (fw_dma_raw() != FW_CFG_DMA_MAGIC) {
    panic("QEMU FW_CFG features");
  }
}

static struct FWCfgFile readCfgFile(void) {
  struct FWCfgFile file = {0, 0, 0, {0}};
  file.size             = FW_GET_DAT(32);
  file.select           = FW_GET_DAT(16);
  file.reserved         = FW_GET_DAT(16);

  for (int i = 0; i < 7; i++) {
    ((u64*)&file.name)[i] = fw_dat_raw();
  }
  return file;
}

static struct FWCfgFiles* readCfgFiles(void) {
  fw_select(FW_CFG_FILE_DIR);
  u32 count = FW_GET_DAT(32);
  if (sizeof(struct FWCfgFiles) + count * sizeof(struct FWCfgFile) > PGSIZE) {
    panic("FW_CFG_FILE_DIR toomany");
  }
  struct FWCfgFiles* files = kalloc();
  files->count             = count;
  for (u32 i = 0; i < count; i++) {
    files->f[i] = readCfgFile();
  }
  return files;
}

u8* readFileEntry(struct FWCfgFile* file) {
  if (file->size >= PGSIZE) {
    return 0;
  }
  u64* data = kalloc();
  fw_select(file->select);
  for (u32 i = 0; i < file->size; i += sizeof(u64)) {
    data[i] = fw_dat_raw(); // overreads 0's?
  }
  ((u8*)data)[file->size] = '\0';
  return (u8*)data;
}

void dtbinit(void) {
  fw_verify();
  fw_dir = readCfgFiles();
  printf("fw_dir->count = %d\n", fw_dir->count);
  for (u32 i = 0; i < fw_dir->count; i++) {
    struct FWCfgFile* file = &fw_dir->f[i];
    printf("    %s: %d bytes\n", file->name, file->size);
  }
}
