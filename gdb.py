from __future__ import print_function
import gdb
from gdb import Command

MASK_64 = 0xffffffffffffffff

class PageTable(Command):
    def __init__(self):
        super(PageTable, self).__init__("pagetable", gdb.COMMAND_DATA)

    def invoke(self, arg, from_tty):
        try:
            self.sub_invoke(arg, from_tty)
        except KeyboardInterrupt:
            pass

    def sub_invoke(self, arg, from_tty):
        args = [x for x in arg.split(' ') if len(x) > 0]
        table_base = 0xfffffffffffff000

        if len(args) > 0:
            try:
                table_base = int(args[0], 0)
            except ValueError:
                print('argument must be an integer')
                return

        table_base = (table_base >> 3) & MASK_64
        table_base = (table_base << 3) & MASK_64

        if table_base & 0xfff != 0:
            table_base = table_base & ~0xfff
            print('WARN: table misaligned, masking to {:#018x}'.format(table_base))

        mask = 0xff80000000000000  # high 9 bits
        tbl_wrk = ((table_base & 0x000ffffffffff000) << 16) & MASK_64

        table_level = 4
        # noinspection PyUnresolvedReferences
        for i in xrange(4):
            if mask & ~tbl_wrk != 0:
                table_level = i
                break

            tbl_wrk = (tbl_wrk << 9) & MASK_64

        print('inferred table level: {}'.format(table_level))

        table = gdb.parse_and_eval('{:#x} as *const u64'.format(table_base))

        entries_written = False

        # noinspection PyUnresolvedReferences
        for i in xrange(512):
            entry = table[i]
            if entry != 0:
                entries_written = True

                print()

                entry_addr = table_base + i*8
                print('entry {} [{:#x}]'.format(i, entry_addr))

                print('\traw: {:#018x}'.format(int(entry)))

                flags = []
                if entry & 1 << 0 != 0:
                    flags.append('PRESENT')

                if entry & 1 << 1 != 0:
                    flags.append('WRITABLE')

                if entry & 1 << 2 != 0:
                    flags.append('USER_ACCESS')

                if entry & 1 << 3 != 0:
                    flags.append('WRITETHROUGH')

                if entry & 1 << 4 != 0:
                    flags.append('NOCACHE')

                if entry & 1 << 5 != 0:
                    flags.append('ACCESSED')

                if entry & 1 << 6 != 0:
                    flags.append('DIRTY')

                huge = entry & 1 << 7 != 0
                if huge:
                    flags.append('HUGE')

                if entry & 1 << 8 != 0:
                    flags.append('GLOBAL')

                if entry & 1 << 63 != 0:
                    flags.append('NX')

                # TODO: check remaining flags for validity (see intel vol. 4 ch. 4 table 4-9)

                print('\tflags: {}'.format(' '.join(flags)))

                frame = (entry & 0x000ffffffffff000) >> 12

                if table_level > 1 and not huge:
                    subtable_addr = (table_base << 9 | i << 12) & MASK_64
                    print('\tpointed table:')

                    subtable_vmem_base = (subtable_addr << (16 + 9 * (table_level - 1)) & MASK_64) >> 16
                    if subtable_vmem_base & (1 << 47) != 0:
                        subtable_vmem_base |= 0xffff000000000000

                    subtable_size = 1 << (12 + (table_level - 1) * 9)

                    print('\t\tvmem range: {:#x} - {:#x}'.format(subtable_vmem_base, subtable_vmem_base + subtable_size - 1))

                    print('\t\tframe: {:#x}'.format(int(frame)))
                    print('\t\taddr: {:#018x}'.format(subtable_addr))

                elif table_level == 4 or (table_level == 1 and huge):
                    print('\tERROR: unsupported HUGE flag (only applies to levels 2 and 3)')

                else:
                    entry_mag = (12 + (table_level - 1) * 9)
                    virt_base = (entry_addr << (16 + 9 * table_level) & MASK_64) >> 16

                    if virt_base & (1 << 47) != 0:
                        virt_base |= 0xffff000000000000

                    size = 1 << entry_mag

                    print('\taddressing characteristics:')
                    print('\t\tvmem: {:#x} - {:#x}'.format(virt_base, virt_base + size - 1))

                    frame_count = size >> 12

                    if frame_count > 1:
                        print('\t\tframes: {:#x} - {:#x}'.format(int(frame), int(frame) + (size >> 12) - 1))
                    else:
                        print('\t\tframe: {:#x}'.format(int(frame)))

                    print('\t\tsize: {:#x} bytes ({} {})'.format(size, size >> 12, 'frame' if frame_count == 1 else 'frames'))

        if not entries_written:
            print('\tno entries')


PageTable()
