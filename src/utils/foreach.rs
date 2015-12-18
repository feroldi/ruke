use core::iter::Iterator;

pub trait Foreach<T> {
    fn foreach<F>(mut self, mut f: F) where F: FnMut(T);
}

impl<T, I: Iterator<Item=T>> Foreach<T> for I {
    fn foreach<F>(mut self, mut f: F) where F: FnMut(T) {
        while let Some(item) = self.next() {
            f(item);
        }
    }
}

