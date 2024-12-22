#include "kernel/types.h"

usize strlen(const char* str) {
  usize len;
  for (len = 0; str[len] != '\0'; len++) {
  }
  return len;
}

void memset(void* ptr, u8 byte, usize size) {
  u8* bytes = ptr;
  for (usize i = 0; i < size; i++) {
    bytes[i] = byte;
  }
}

void memmove(void* dst, const void* src, usize size) {
  u8*       dst_ = dst;
  const u8* src_ = src;
  for (usize i = 0; i < size; i++) {
    dst_[i] = src_[i];
  }
}

char* strcpy(char* dest, const char* src) {
  usize len;
  for (len = 0; src[len] != '\0'; len++) {
    dest[len] = src[len];
  }
  dest[len] = '\0';
  return dest;
}

char* strncpy(char* dest, const char* src, usize size) {
  usize len;
  for (len = 0; src[len] != '\0' && len < size; len++) {
    dest[len] = src[len];
  }
  if (len < size) {
    dest[len] = '\0';
  }
  return dest;
}

char* safestrcpy(char* dest, const char* src, usize size) {
  if (size == 0) {
    return dest;
  }
  dest[size - 1] = '\0';
  return strncpy(dest, src, size - 1);
}
