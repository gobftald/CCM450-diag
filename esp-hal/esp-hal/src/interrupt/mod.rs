#[cfg(riscv)]
mod riscv;

#[no_mangle]
// 88
//extern "C" fn EspDefaultHandler(_interrupt: crate::peripherals::Interrupt) {
extern "C" fn EspDefaultHandler() {
    //panic!("Unhandled interrupt: {:?}", _interrupt);
    panic!("Unhandled interrupt");
}
