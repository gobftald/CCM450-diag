.core : ALIGN(4)
{
    
    *(.text.*alloc*raw_vec*finish_grow*)
    *(.text.*alloc*raw_vec*RawVecInner*try_allocate_in*)
    *(.text.*alloc*raw_vec*RawVec*grow_one*)

    *(.text.*alloc*vec*from_elem*)
    *(.text.*alloc*vec*Vec*as*core*ops*drop*Drop*drop*)

    *(.text.*alloc*slice*impl*alloc*borrow*ToOwned*for*to_owned*)

    *(.text.*alloc*collections*vec_deque*VecDeque*push_back*)    
    *(.text.*alloc*collections*vec_deque*VecDeque*grow*)

    *(.text.*___rustc14___rust_realloc)


    *(.text.*core*ops*function*FnOnce*call_once*vtable.shim*)
    *(.text.*core*ops*function*impls*impl*core*ops*function*FnMut*for*mut*call_mut*)

    *(.text.*core*cmp*PartialEq*ne*)

    *(.text.*core*slice*index*slice_index_fail*)
    *(.text.*core*slice*memchr*memchr_aligned*)

    *(.text.*core*cell*once*OnceCell*try_init*)

    *(.text.*core*future*poll_fn*PollFn*as*core*future*future*Future*poll*)
    
    *(.text.*core*ffi*c_str*CStr*to_str*)


    *(.text.*compiler_builtins*mem*strlen*)
    *(.text.strlen)

} > ROTEXT