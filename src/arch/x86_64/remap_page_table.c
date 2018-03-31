#include "multiboot2.h"
#include <elf.h>
#include <sys/types.h>

#define BOOT __attribute__((section(".boot_text")))
#define BOOT_STATIC __attribute__((section(".boot_data"))) static
#define bool uint8_t
#define u8 uint8_t
#define u32 uint32_t
#define u64 uint64_t
#define u16 uint16_t

BOOT u64 entry_for_frame(u64 frame_idx) {
    return (frame_idx << 12) | 0x3; // present and writable
}

BOOT u64 p4_index(u64 page_idx) {
    return (page_idx >> 27) & 0x1ff;
}

BOOT u64 p3_index(u64 page_idx) {
    return (page_idx >> 18) & 0x1ff;
}

BOOT u64 p2_index(u64 page_idx) {
    return (page_idx >> 9) & 0x1ff;
}

BOOT u64 p1_index(u64 page_idx) {
    return (page_idx >> 0) & 0x1ff;
}

BOOT u64* p4_table() {
    return (u64*)0xfffffffffffff000;
}

BOOT u64* p3_table(u64 page_idx) {
    return (u64*)(((u64)p4_table() << 9) | (p4_index(page_idx) << 12));
}

BOOT u64* p2_table(u64 page_idx) {
    return (u64*)(((u64)p3_table(page_idx) << 9) | (p3_index(page_idx) << 12));
}

BOOT u64* p1_table(u64 page_idx) {
    return (u64*)(((u64)p2_table(page_idx) << 9) | (p2_index(page_idx) << 12));
}

BOOT void clear_page(u64* ptr) {
    for (int i = 0; i < 4096/8; i++) {
        ptr[i] = 0;
    }
}

#define BUFFER_HEIGHT 25
#define BUFFER_WIDTH 80

BOOT_STATIC u16* VGA_BUF = (u16*)0xb8000;

BOOT void clear_screen() {
    for (int i = 0; i < BUFFER_HEIGHT; i++) {
        for (int j = 0; j < BUFFER_WIDTH; j++) {
            VGA_BUF[i * BUFFER_WIDTH + j] = 0;
        }
    }
}

BOOT void message(const char* s, u16 color, u8 x, u8 y) {
    u16* vga_idx = VGA_BUF + y * BUFFER_WIDTH + x;
    while (*s) *vga_idx++ = ((u16)*s++) | color << 8;
}

BOOT void panic(const char* s) {
    BOOT_STATIC char error[] = "PANIC: ";
    message(error, 0x04, 6, 11);
    message(s, 0x0f, 6 + sizeof(error) - 1, 11);
    __asm__("hlt");
}

BOOT void remap_page_tables(void* addr) {
    clear_screen();

    struct multiboot_tag* tag;

    // TODO: make sure a) we have enough memory, and b) that frames don't collide
    // 512 2MB frames from boot mapping should take up 0x19000 4KB frames
    u64 tmp_alloc_frame_idx = 0xf0000;

    for (tag = (struct multiboot_tag *) (addr + 8);
         tag->type != MULTIBOOT_TAG_TYPE_END;
         tag = (struct multiboot_tag *) ((multiboot_uint8_t *) tag + ((tag->size + 7) & ~7))) {

         switch (tag->type) {
         case MULTIBOOT_TAG_TYPE_ELF_SECTIONS: ;
            struct multiboot_tag_elf_sections* sects = (struct multiboot_tag_elf_sections*) tag;

            for (u32 i = 0; i < sects->num; i++) {
                Elf64_Shdr* hdr = (Elf64_Shdr*)(sects->sections + i*sects->entsize);

                bool valid = (hdr->sh_flags & SHF_ALLOC) && (hdr->sh_size > 0) && !(hdr->sh_type & SHT_NULL);
                if (!valid) continue;

                u64 virt_addr = hdr->sh_addr;
                u64 base_page = virt_addr >> 12;
                u64 phys_addr = hdr->sh_offset;
                u64 phys_frame = phys_addr / 4096;
                u64 size = hdr->sh_size;
                u64 frame_count = size / 4096;

                // lower half is boot-related; this should already have been mapped
                if (virt_addr < 0xffff800000000000) continue;

                if (size % 4096 != 0 || phys_addr % 4096 != 0) {
                    BOOT_STATIC char message[] = "bad align";
                    panic(message);
                }

                if (hdr->sh_type & SHT_NOBITS) { // BSS: allocate
                    phys_frame = tmp_alloc_frame_idx;
                    tmp_alloc_frame_idx += size / 4096; // relying on alignment here
                }

                for (int j = 0; j < frame_count; j++) {
                    u64 page_idx = base_page + j;

                    // p4 and p3 should be mapped already. just crash if they're not.

                    u64* p4_ent = &p4_table()[p4_index(page_idx)];
                    if (!*p4_ent) {
                        BOOT_STATIC char message[] = "p4 unmapped";
                        panic(message);
                    }

                    volatile u64* p3_ent = &(p3_table(page_idx)[p3_index(page_idx)]);
                    if (!*p3_ent) {
                        BOOT_STATIC char message[] = "p3 unmapped";
                        panic(message);
                    }

                    volatile u64* p2_ent = &(p2_table(page_idx)[p2_index(page_idx)]);
                    if (!*p2_ent) {
                        *p2_ent = entry_for_frame(phys_frame) | (1 << 7); // giant page
                    }
                }
            }

            break;

         default:
            break;
         }
    }
}

