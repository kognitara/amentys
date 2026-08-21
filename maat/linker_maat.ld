ENTRY(_start)

SECTIONS {
    /* L'espace utilisateur commence classiquement à 4 Mo (0x400000) */
    . = 0x400000;

    .text : ALIGN(4K) { *(.text .text.*) }
    .rodata : ALIGN(4K) { *(.rodata .rodata.*) }
    .data : ALIGN(4K) { *(.data .data.*) }
    .bss : ALIGN(4K) { *(.bss .bss.*) }
}