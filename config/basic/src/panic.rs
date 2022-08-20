// use sbi_rt::{system_reset, RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE};
// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
//     unreachable!()
// }

use sbi_rt::{self, RESET_REASON_NO_REASON,RESET_TYPE_SHUTDOWN};
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        output::println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        output::println!("Panicked: {}", info.message().unwrap());
    }
    sbi_rt::system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}