ENTRY(_start)
SECTIONS
{
    . = 0x80000;

	__ro_start = .;
    .text : {
        *(.text._start)
        *(.text*)
    }
	.expection_vectors : { *(.expection_vectors*) }
    .rodata : { *(.rodata*) }
	. = ALIGN(65536);
	__ro_end = .;

    .data : { *(.data*) }

    .bss ALIGN(8) :
    {
        __bss_start = .;
        *(.bss*)
		. = ALIGN(8);
        __bss_end = .;
    }

	__stack_start = .;
	. = . + 0x2000;
	__stack_end = .;

	/DISCARD/ : { *(.comment*) }
}
