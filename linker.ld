OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    /* Adresse de départ standard pour le code-model kernel en x86_64 */
    . = 0xffffffff80000000;

    .text : ALIGN(0x1000) {
        *(.text .text.*)
    }

    .rodata : ALIGN(0x1000) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(0x1000) {
        *(.data .data.*)
    }

    .bss : ALIGN(0x1000) {
        *(.bss .bss.*)
        *(COMMON)
    }
}
