use crate::arch::traits::*;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::arch::EarlyLogger::_panic(format_args!("{}", info))
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (<$crate::arch::EarlyLogger as $crate::arch::LoggerTrait>::_print(format_args!($($arg)*)));
}
