pub use self::area_frame_allocator::{AreaFrameAllocator};
pub use self::paging::test_paging;

use self::paging::PhysicalAddr;

pub mod area_frame_allocator;
pub mod paging;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn containing_addr(addr: usize) -> Frame {
        Frame {
            number: addr / PAGE_SIZE,
        }
    }

    fn start_addr(&self) -> PhysicalAddr {
        self.number * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}


