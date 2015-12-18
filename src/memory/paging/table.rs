use core::ops::{Index, IndexMut};
use core::marker::PhantomData;

use utils::Foreach;
use memory::paging::entry::*;
use memory::paging::ENTRY_COUNT;
use memory::FrameAllocator;

pub const PAGE4: *mut Table<Level4> = 0xfffffffffffff000 as *mut _;

pub trait TableLevel {}
pub trait HierachicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

pub enum Level4 {}
enum Level3 {}
enum Level2 {}
enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

impl HierachicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierachicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierachicalLevel for Level2 {
    type NextLevel = Level1;
}

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L: TableLevel> Index<usize> for Table<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for Table<L> {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L: TableLevel> Table<L> {
    pub fn zero(&mut self) {
        self.entries
            .iter_mut()
            .foreach(|entry| entry.set_unused());
    }
}

impl<L: HierachicalLevel> Table<L> {
    fn next_table_addr(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
            let table_addr = self as *const _ as usize;
            Some((table_addr << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }
    
    pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A)
                                -> &mut Table<L::NextLevel>
        where A: FrameAllocator
    {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(HUGE_PAGE),
                    "mapping code does not support huge pages");
            let frame = allocator.allocate_frame().expect("no frames available");
            self.entries[index].set(frame, PRESENT | WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}

