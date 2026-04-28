/*
 * Compatibility shim for microsoft/mu_msvm UEFI headers.
 * Provides freestanding type definitions so the headers compile
 * without UEFI build infrastructure.
 *
 * The original headers are from microsoft/mu_msvm and are licensed
 * under BSD-2-Clause-Patent.
 */

#ifndef _MSVM_COMPAT_H
#define _MSVM_COMPAT_H

/* Freestanding integer types (replace UEFI BaseTypes.h) */
typedef unsigned char UINT8;
typedef unsigned char BOOLEAN;
typedef unsigned short UINT16;
typedef unsigned int UINT32;
typedef unsigned long long UINT64;
typedef int INT32;

/* Stub out UEFI build helpers */
#ifndef STATIC_ASSERT_1
#define STATIC_ASSERT_1(expr)
#endif

#ifndef OFFSET_OF
#define OFFSET_OF(type, field) __builtin_offsetof(type, field)
#endif

#ifndef __packed
#define __packed __attribute__((packed))
#endif

#endif /* _MSVM_COMPAT_H */
