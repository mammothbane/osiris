#include "multiboot2.h"
#include <elf.h>
#include <sys/types.h>

#define BOOT __attribute__((section(".boot_text")))
#define bool uint8_t
#define u32 uint32_t
#define u64 uint64_t

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

extern void alloc_frame(u64 page_idx, u64 frame_idx);

BOOT void remap_page_tables(void* addr) {
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
                    continue;
                    // panic-- we're not aligned
                }

                if (hdr->sh_type & SHT_NOBITS) { // BSS: allocate
                    phys_frame = tmp_alloc_frame_idx;
                    tmp_alloc_frame_idx += size / 4096; // relying on alignment here
                }

                for (int j = 0; j < frame_count; j++) {
                    u64 page_idx = base_page + j;

//                    alloc_frame(page_idx, phys_frame + j);

                    u64* p4_ent = &p4_table()[p4_index(page_idx)];
                    if (!*p4_ent) {
                        *p4_ent = entry_for_frame(tmp_alloc_frame_idx++);
                        clear_page(p3_table(page_idx));
                    }

                    __asm__("mov %%cr3, %%rax\n"
                            "mov %%rax, %%cr3"
                            :
                            :
                            : "rax");
                    __asm__("invd");

                    volatile u64* p3_ent = &(p3_table(page_idx)[p3_index(page_idx)]);
                    if (!*p3_ent) {
                        *p3_ent = entry_for_frame(tmp_alloc_frame_idx++);
                        clear_page(p2_table(page_idx));
                    }

                    __asm__("mov %%cr3, %%rax\n"
                            "mov %%rax, %%cr3"
                            :
                            :
                            : "rax");
                    __asm__("invd");

                    volatile u64* p2_ent = &(p2_table(page_idx)[p2_index(page_idx)]);
                    if (!*p2_ent) {
                        *p2_ent = entry_for_frame(tmp_alloc_frame_idx++);
                        clear_page(p1_table(page_idx));
                    }

                    __asm__("mov %%cr3, %%rax\n"
                            "mov %%rax, %%cr3"
                            :
                            :
                            : "rax");
                    __asm__("invd");

                    volatile u64* p1_ent = &(p1_table(page_idx)[p1_index(page_idx)]);
                    if (!*p1_ent) {
                        *p1_ent = entry_for_frame(phys_frame + j);
                        // fine to leave garbage here
                    } else {
                        // panic?
                    }
                }
            }

            break;

         default:
            break;
         }
    }
}

