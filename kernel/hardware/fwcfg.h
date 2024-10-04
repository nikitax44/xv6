#include "defs.h"
#include "memlayout.h"
#include "types.h"

struct FWCfgFile { /* an individual file entry, 64 bytes total */
  u32  size;       /* size of referenced fw_cfg item, big-endian */
  u16  select;     /* selector key of fw_cfg item, big-endian */
  u16  reserved;
  char name[56]; /* fw_cfg item name, NUL-terminated ascii */
};

struct FWCfgFiles {       /* the entire file directory fw_cfg item */
  u32              count; /* number of entries, in big-endian format */
  struct FWCfgFile f[];   /* array of file entries, see below */
};

extern struct FWCfgFiles* fw_dir;

u8*  readFileEntry(struct FWCfgFile* file);
void fw_dump(void);
