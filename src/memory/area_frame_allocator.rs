
use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    // A simple counter that is increased every time we return a frame.
    // It's initialized to 0 and every frame below it counts as used.
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        uninplemented!()
    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}

// This function chooses the area with the minimal base address that still has free frames, i.e.
// next_free_frame is smaller than its last frame. Note that we need to clone the iterator because
// the min_by function consumes it. If there are no areas with free frames left, min_by
// automatically returns the desired None.
fn choose_next_area(&mut self) {
    self.current_area = self.areas.clone().filter(|area| {
    let address = area.base_addr + area.length - 1;
        Frame::containing_address(address as usize) >= self.next_free_frame
    }).min_by(|area| area.base_addr);

    if let Some(area) = self.current_area {
        let start_frame = Frame::containing_address(area.base_addr as usize);
        if self.next_free_frame < start_frame {
            self.next_free_frame = start_frame;
        }
    }
}
