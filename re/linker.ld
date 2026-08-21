ENTRY(_start)

PHDRS
{
    limine_requests PT_LOAD FLAGS(4); /* Read Only pour les requêtes */
    text            PT_LOAD FLAGS(5); /* Read + Execute */
    rodata          PT_LOAD FLAGS(4); /* Read Only */
    data            PT_LOAD FLAGS(6); /* Read + Write */
}

SECTIONS
{
    /* Adresse de la moitié supérieure (Higher Half) requise */
    . = 0xffffffff80000000;

    /* CRUCIAL POUR LIMINE v12.x : Section dédiée aux requêtes */
    .limine_requests : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :limine_requests

    . = ALIGN(0x1000);

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(0x1000);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /* Supprime les sections inutiles pour un kernel propre */
    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
