.rwtext : ALIGN(4)
{
    . = ALIGN (4);

    *(.rwtext.literal .rwtext .rwtext.literal.* .rwtext.*)

    . = ALIGN(4);

} > RWTEXT