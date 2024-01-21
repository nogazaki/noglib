//! Free memory block header

use core::ptr;

/// An intrusive doubly linked list
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct BlockHeader {
    /// Pointer to previous header in the list
    prev: *mut BlockHeader,
    /// Pointer to next header in the list
    next: *mut BlockHeader,
}

impl BlockHeader {
    /// Create a new header
    pub(crate) const fn new() -> Self {
        BlockHeader {
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    /// Add a node to the list of header
    ///
    /// # Safety
    /// `node` must not be null pointer and is properly aligned
    pub(crate) unsafe fn push(&mut self, node: *mut BlockHeader) {
        (*node).next = self.next;
        (*node).prev = self;

        if !self.next.is_null() {
            (*self.next).prev = node;
        }
        self.next = node;
    }

    /// Attempt to remove the next header from the list
    pub(crate) fn pop_next(&mut self) -> Option<*mut BlockHeader> {
        if self.is_tail() {
            return None;
        }

        let node = self.next;
        // SAFETY: all pointer are guarantee through `push`
        unsafe {
            self.next = (*node).next;
            if !self.next.is_null() {
                (*self.next).prev = self;
            }
            (*node) = BlockHeader::new();
        }

        Some(node)
    }

    /// Attempt to remove this header from the list
    pub(crate) fn pop(&mut self) -> *mut BlockHeader {
        if !self.prev.is_null() {
            unsafe { (*self.prev).next = self.next }
        }
        if !self.next.is_null() {
            unsafe { (*self.next).prev = self.prev }
        }

        *self = BlockHeader::new();
        self
    }

    /// Return `true` if the list ended with this header
    #[inline]
    pub(crate) fn is_tail(&self) -> bool {
        self.next.is_null()
    }

    /// Return an mutable iterator over the headers in the list
    #[inline]
    pub(crate) fn iter_mut(&mut self) -> Iter {
        Iter { node: self }
    }
}

/// An iterator over the linked list
pub(crate) struct Iter {
    /// Current header
    node: *mut BlockHeader,
}
impl Iterator for Iter {
    type Item = *mut BlockHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.node.is_null() {
            None
        } else {
            let node = self.node;
            let next = unsafe { (*self.node).next };
            self.node = next;

            Some(node)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::shadow_unrelated)]
    fn test_head_list() {
        let mut main_node = BlockHeader::new();
        let mut node_1 = BlockHeader::new();
        let mut node_2 = BlockHeader::new();

        /* `main_node` */
        assert!(main_node.prev.is_null());
        assert!(main_node.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());
        assert!(node_2.prev.is_null());
        assert!(node_2.next.is_null());

        /* `main_node` -> `node_2` */
        unsafe { main_node.push(&mut node_2) };
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut main_node as *mut _);
        assert!(node_2.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());

        /* `main_node` -> `node_1` -> `node_2` */
        unsafe { main_node.push(&mut node_1) };
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_1 as *mut _);
        assert_eq!(node_1.prev, &mut main_node as *mut _);
        assert_eq!(node_1.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut node_1 as *mut _);
        assert!(node_2.next.is_null());

        /* `main_node` -> `node_2` */
        let popped = main_node.pop_next();
        assert!(popped.is_some_and(|ptr| ptr == &mut node_1 as *mut _));
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut main_node as *mut _);
        assert!(node_2.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());

        /* `main_node` */
        let popped = main_node.pop_next();
        assert!(popped.is_some_and(|ptr| ptr == &mut node_2 as *mut _));
        assert!(main_node.prev.is_null());
        assert!(main_node.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());
        assert!(node_2.prev.is_null());
        assert!(node_2.next.is_null());

        let popped = main_node.pop_next();
        assert!(popped.is_none());

        /* `main_node` -> `node_1` */
        unsafe { main_node.push(&mut node_1) };
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_1 as *mut _);
        assert_eq!(node_1.prev, &mut main_node as *mut _);
        assert!(node_1.next.is_null());
        assert!(node_2.prev.is_null());
        assert!(node_2.next.is_null());

        /* `main_node` -> `node_1` -> `node_2` */
        unsafe { node_1.push(&mut node_2) }
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_1 as *mut _);
        assert_eq!(node_1.prev, &mut main_node as *mut _);
        assert_eq!(node_1.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut node_1 as *mut _);
        assert!(node_2.next.is_null());

        /* `main_node` -> `node_2` */
        let popped = node_1.pop();
        assert_eq!(popped, &mut node_1 as *mut _);
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut main_node as *mut _);
        assert!(node_2.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());

        /* `main_node` */
        let popped = node_2.pop();
        assert_eq!(popped, &mut node_2 as *mut _);
        assert!(main_node.prev.is_null());
        assert!(main_node.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());
        assert!(node_2.prev.is_null());
        assert!(node_2.next.is_null());
    }

    #[test]
    fn test_iter() {
        let mut main_node = BlockHeader::new();
        let mut node_1 = BlockHeader::new();
        let mut node_2 = BlockHeader::new();

        /* `main_node` -> `node_1` -> `node_2` */
        unsafe { main_node.push(&mut node_2) };
        unsafe { main_node.push(&mut node_1) };
        assert!(main_node.prev.is_null());
        assert_eq!(main_node.next, &mut node_1 as *mut _);
        assert_eq!(node_1.prev, &mut main_node as *mut _);
        assert_eq!(node_1.next, &mut node_2 as *mut _);
        assert_eq!(node_2.prev, &mut node_1 as *mut _);
        assert!(node_2.next.is_null());

        let main_node_address = &mut main_node as *mut _;
        let mut iterator = main_node.iter_mut();
        assert!(iterator.next().is_some_and(|ptr| ptr == main_node_address));
        assert!(iterator.next().is_some_and(|ptr| ptr == &mut node_1));
        assert!(iterator.next().is_some_and(|ptr| ptr == &mut node_2));
        assert!(iterator.next().is_none());

        for node in main_node.iter_mut().skip(1) {
            unsafe { (*node).pop() };
        }

        /* `main_node` */
        assert!(main_node.prev.is_null());
        assert!(main_node.next.is_null());
        assert!(node_1.prev.is_null());
        assert!(node_1.next.is_null());
        assert!(node_2.prev.is_null());
        assert!(node_2.next.is_null());
    }
}
