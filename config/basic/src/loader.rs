use crate::APP_BASE_ADDRESS;

#[repr(C)]
pub struct AppMeta {
    base: u64,
    step: u64,
    count: u64,
    first: u64,
}
use crate::APP_SIZE_LIMIT;
impl AppMeta {
    pub unsafe fn load(&self, i: usize) -> usize {
        // get apps location list
        let slice = core::slice::from_raw_parts(
            &self.first as *const _ as *const usize,
            (self.count + 1) as _,
        );
        // find the location and count for it's address

        let pos = slice[i];
        let size = slice[i + 1] - pos;
        let base = APP_BASE_ADDRESS;
        // let base = self.base as usize + i * self.step as usize;

        core::ptr::copy_nonoverlapping::<u8>(pos as _, base as _, size);
        // 这个地方根据ydrMaster提出的，清零其他需要用到的区域的说法，我感觉不大需要在我的程序中用到
        // 同时我也感觉他的这整个程序有点类似于
        core::slice::from_raw_parts_mut(base as *mut u8, APP_SIZE_LIMIT)[size..].fill(0);
        base
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count as _
    }
}