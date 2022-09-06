
#[repr(C)]
pub struct AppMeta {
    pub base: u64,
    pub step: u64,
    pub count: u64,
    first: u64,
}
use crate::APP_SIZE_LIMIT;
use output::log::*;
impl AppMeta {
    pub unsafe fn show_origin_load_info(&self) {
        let slice = core::slice::from_raw_parts(
            &self.first as *const _ as *const usize,
            (self.count + 1) as usize,
        );
        for i in 0..slice.len()-1 {
            info!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                slice[i],
                slice[i + 1]
            );
        }
    }

    pub unsafe fn load(&self, i: usize, step: usize) -> usize {
        // get apps location list
        let slice = core::slice::from_raw_parts(
            &self.first as *const _ as *const usize,
            (self.count + 1) as _,
        );
        // find the location and count for it's address

        #[allow(unused_imports)]
        use crate::APP_BASE_ADDRESS;
        let pos = slice[i];
        let size = slice[i + 1] - pos;
        // let base = APP_BASE_ADDRESS;
        let base = self.base as usize + i * step as usize;
        core::ptr::copy_nonoverlapping::<u8>(pos as _, base as _, size);
        core::slice::from_raw_parts_mut(base as *mut u8, APP_SIZE_LIMIT)[size..].fill(0);
        base
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count as _
    }
}