use core::ptr::Unique;

pub use self::entry::*;
use memory::{PAGE_SIZE, Frame, FrameAllocator};
use self::table::{Table, Level4};

pub mod entry;
pub mod table;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddr = usize;
pub type VirtualAddr = usize;

trait PageIndex {
    fn shift() -> usize;
}

enum Page4 {}
enum Page3 {}
enum Page2 {}
enum Page1 {}

impl PageIndex for Page4 {
    fn shift() -> usize { 27 }
}

impl PageIndex for Page3 {
    fn shift() -> usize { 18 }
}

impl PageIndex for Page2 {
    fn shift() -> usize { 9 }
}

impl PageIndex for Page1 {
    fn shift() -> usize { 0 }
}

pub struct Page {
    number: usize,
}

impl Page {
    fn containing_addr(addr: VirtualAddr) -> Page {
        assert!(addr < 0x0000_8000_0000_0000 || addr >= 0xffff_8000_0000_0000,
                "invalid address: 0x{:x}", addr);
        Page { number: addr / PAGE_SIZE }
    }

    fn start_addr(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn page_index<P: PageIndex>(&self) -> usize {
        (self.number >> P::shift()) & 0x1ff
    }
}

pub struct RecursivePageTable {
    p4: Unique<Table<Level4>>,
}

impl RecursivePageTable {
    pub unsafe fn new() -> RecursivePageTable {
        RecursivePageTable {
            p4: Unique::new(table::PAGE4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }

    pub fn translate(&self, virtual_addr: VirtualAddr) -> Option<PhysicalAddr> {
        let offset = virtual_addr % PAGE_SIZE;
        self.translate_page(Page::containing_addr(virtual_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;

        let p3 = self.p4().next_table(page.page_index::<Page4>());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.page_index::<Page3>()];
                
                // is it 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.page_index::<Page2>() *
                                    ENTRY_COUNT + page.page_index::<Page1>(),
                        });
                    }
                }

                if let Some(p2) = p3.next_table(page.page_index::<Page3>()) {
                    let p2_entry = &p2[page.page_index::<Page2>()];
                    // is it 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.page_index::<Page1>(),
                            })
                        }
                    }
                }

                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.page_index::<Page3>()))
          .and_then(|p2| p2.next_table(page.page_index::<Page2>()))
          .and_then(|p1| p1[page.page_index::<Page1>()].pointed_frame())
          .or_else(huge_page)
    }


    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let p4 = self.p4_mut();
        let mut p3 = p4.next_table_create(page.page_index::<Page4>(), allocator);
        let mut p2 = p3.next_table_create(page.page_index::<Page3>(), allocator);
        let mut p1 = p2.next_table_create(page.page_index::<Page2>(), allocator);

        assert!(p1[page.page_index::<Page1>()].is_unused());
        p1[page.page_index::<Page1>()].set(frame, flags | PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator);
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing_addr(frame.start_addr());
        self.map_to(page, frame, flags, allocator);
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        // Ensures that the page is mapped.
        assert!(self.translate(page.start_addr()).is_some());

        let p1 = self.p4_mut()
                     .next_table_mut(page.page_index::<Page4>())
                     .and_then(|p3| p3.next_table_mut(page.page_index::<Page3>()))
                     .and_then(|p2| p2.next_table_mut(page.page_index::<Page2>()))
                     .expect("mapping code doesn't suport huge pages");
        let frame = p1[page.page_index::<Page1>()].pointed_frame().unwrap();
        p1[page.page_index::<Page1>()].set_unused();

        // flushes the translation lookaside buffer (TLB) cache
        unsafe { ::x86::tlb::flush(page.start_addr()) };

        // TODO free p{1, 2, 3} table if empty
        //allocator.deallocate_frame(frame);
    }
}


pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator
{
    let mut page_table = unsafe { RecursivePageTable::new() };

    // address 0 is mapped
    println!("Some = {:?}", page_table.translate(0));
    // second P1 entry
    println!("Some = {:?}", page_table.translate(4096));
    // second P2 entry
    println!("Some = {:?}", page_table.translate(512 * 4096));
    // 300th P2 entry
    println!("Some = {:?}", page_table.translate(300 * 512 * 4096));
    // second P3 entry
    println!("None = {:?}", page_table.translate(512 * 512 * 4096));
    // last mapped byte
    println!("Some = {:?}", page_table.translate(512 * 512 * 4096 - 1));

    // testing mapping

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_addr(addr);
    let frame = allocator.allocate_frame().expect("no more frames");
    println!("None = {:?}, map to {:?}",
             page_table.translate(addr),
             frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.allocate_frame());

    println!("{:#x}", unsafe { *(Page::containing_addr(addr).start_addr() as *const u64) });

    // testing unmapping

    page_table.unmap(Page::containing_addr(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));

    // println!("{:#x}", unsafe { *(Page::containing_addr(addr).start_addr() as *const u64) });
    // ^ ERROR: causes a page fault
}

