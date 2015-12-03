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
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            //let frame = self.next_free_frame.clone();
            let frame = Frame { number: self.next_free_frame.number };
            
            let current_area_last_frame = {
                let addr = area.base_addr + area.length - 1;
                Frame::containing_addr(addr as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1,
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1,
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }

            self.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        unimplemented!()
    }

    // This function chooses the area with the minimal base address that still has free frames, i.e.
    // next_free_frame is smaller than its last frame. Note that we need to clone the iterator because
    // the min_by function consumes it. If there are no areas with free frames left, min_by
    // automatically returns the desired None.
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize,
               multiboot_start: usize, multiboot_end: usize,
               memory_areas: MemoryAreaIter) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_addr(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_addr(kernel_start),
            kernel_end: Frame::containing_addr(kernel_end),
            multiboot_start: Frame::containing_addr(multiboot_start),
            multiboot_end: Frame::containing_addr(multiboot_end),
        };

        allocator.choose_next_area();
        allocator
    }

    pub fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let addr = area.base_addr + area.length - 1;
            Frame::containing_addr(addr as usize) >= self.next_free_frame
        }).min_by(|area| area.base_addr);
        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_addr(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
