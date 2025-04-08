/* Do not exceed this mark in the error messages above                                    | */
ASSERT(ORIGIN(ROTEXT) % 4 == 0, "
ERROR(riscv-rt): the start of the ROTEXT must be 4-byte aligned");

ASSERT(ORIGIN(RODATA) % 4 == 0, "
ERROR(riscv-rt): the start of the RODATA must be 4-byte aligned");

ASSERT(_stext % 4 == 0, "
ERROR(riscv-rt): `_stext` must be 4-byte aligned");

ASSERT(_data_start % 4 == 0 && _data_end % 4 == 0, "
BUG(riscv-rt): .data is not 4-byte aligned");

ASSERT(_bss_start % 4 == 0 && _bss_end % 4 == 0, "
BUG(riscv-rt): .bss is not 4-byte aligned");

ASSERT(_stext + SIZEOF(.text) < ORIGIN(ROTEXT) + LENGTH(ROTEXT), "
ERROR(riscv-rt): The .text section must be placed inside the ROTEXT region.
Set _stext to an address smaller than 'ORIGIN(ROTEXT) + LENGTH(ROTEXT)'");
