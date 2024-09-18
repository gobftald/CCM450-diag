SECTIONS {
  /* must be last segment using RWDATA */
  .stack (NOLOAD) : ALIGN(4)
  {
    . = ALIGN (4);
    _stack_end = ABSOLUTE(.);
    _stack_end_cpu0 = ABSOLUTE(.);
  } > RWDATA
}

PROVIDE(_stack_start = ORIGIN(RWDATA) + LENGTH(RWDATA));
PROVIDE(_stack_start_cpu0 = ORIGIN(RWDATA) + LENGTH(RWDATA));
