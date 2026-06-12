use core::cell::Cell;
use core::{cell::UnsafeCell, mem::MaybeUninit};
use core::alloc::Layout;
use alloc::alloc::{alloc, dealloc};

extern crate alloc;

const DEFAULT_CAPACITY: usize = 1024;

#[repr(align(8))]
pub struct Arena<const N: usize = DEFAULT_CAPACITY> {
    container: UnsafeCell<[MaybeUninit<u8>; N]>,
    head: Cell<*mut HeapNode>,
    offset: Cell<usize>,
}

pub struct HeapNode {  //dynamic block linker
    next: *mut HeapNode,
    layout: Layout,
}

pub struct AllocSuccess<'a, T>
where
    T: ?Sized
{
    pub alloc_res: &'a mut T,
    pub alloc_loc: AllocLocation,
}

#[derive(Debug)]
pub enum ArenaError {
    AllocationError,
    AlignmentError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocLocation {
    ArenaAlloc,
    HeapAlloc,
}

// constructors and accessors
impl<const N: usize> Arena<N> {

    // Intialize new arena
    pub fn new() -> Self {

        Self {
            container: UnsafeCell::new([MaybeUninit::uninit(); N]),  
            head: Cell::new(core::ptr::null_mut()),
            offset: Cell::new(0),
        } 

    }

    pub fn capacity(&self) -> usize { N } // Get arena capacity
    pub fn allocated_bytes(&self) -> usize { self.offset.get() } // Get number of bytes allocated in the arena (offset)

    // Add a new node to the heapnode linked list with layout
    unsafe fn update_heapnode_with_layout(&self, head_ptr: *mut HeapNode, new_layout: Layout) {
        unsafe {
            core::ptr::write(head_ptr, HeapNode { 
                next: self.head.get(), 
                layout: new_layout, 
            });
            self.head.set(head_ptr);
        }
    }

}

// direct allocation
impl<const N: usize> Arena<N> {

    // Allocate raw bytes
    pub fn alloc_bytes<'a>(&'a self, size: usize) -> Result<&'a mut [u8], ArenaError> {
        unsafe {
            let cur_offset = self.offset.get();
            let new_offset = cur_offset + size;
            if (new_offset) <= N {
                let start_ptr = self.container.get() as *mut u8;
                self.offset.set(new_offset);
                Ok(core::slice::from_raw_parts_mut(start_ptr.add(cur_offset), size))
            } else {    
                Err(ArenaError::AllocationError)
            }
        }
    }

    // Allocate certain number of bytes with given alignment
    pub fn alloc_align<'a>(&'a self, size: usize, align: usize) -> Result<&'a mut [u8], ArenaError> {

        if align == 0 || (align & (align - 1)) != 0 { return Err(ArenaError::AlignmentError);}

        let cur_addr = unsafe { (self.container.get() as *mut u8).add(self.offset.get()) } as usize;
        let align_addr = (cur_addr + align - 1) & !(align - 1);
        let padding = align_addr - cur_addr;

        let new_offset = self.offset.get() + padding;
        if new_offset + size > N { return Err(ArenaError::AllocationError); }
        self.offset.set(new_offset);
        self.alloc_bytes(size)

    }

    // Free all memory in the arena
    pub fn reset(&mut self) -> () {

        unsafe {

            self.offset.set(0);
            let mut next_ptr = self.head.get();
            let mut cur_ptr = next_ptr;
            while !next_ptr.is_null() {
                next_ptr = (*cur_ptr).next;
                dealloc(cur_ptr as *mut u8, (*cur_ptr).layout);
                cur_ptr = next_ptr;
            }
            self.head.set(core::ptr::null_mut());

        }

    }

}

// slice allocation
impl<const N: usize> Arena<N> {

    // Allocate a slice of bytes
    pub fn alloc_slice_bytes<'a>(&'a self, src: &[u8]) -> Result<&'a mut [u8], ArenaError> {
        let allocated_mem: &mut [u8] = self.alloc_bytes(src.len())?;
        allocated_mem.copy_from_slice(src);
        Ok(allocated_mem)
    }

    // Allocate a string slice
    pub fn alloc_str<'a>(&'a self, slice: &str) -> Result<&'a mut str, ArenaError> {
        let allocated_str_bytes = self.alloc_slice_bytes(slice.as_bytes())?;
        Ok(core::str::from_utf8_mut(allocated_str_bytes).unwrap())
    }

}

// generic allocation
impl<const N: usize> Arena<N> {

    // Allocate space for an object of type T and moves the object into the memory, returns the object back if it values
    pub fn alloc<'a, T>(&'a self, value: T) -> Result<&'a mut T, (ArenaError, T)> {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        let allocated_bytes = self.alloc_align(size, align);
        match allocated_bytes {
            Ok(res) => unsafe { 
                let bytes_ptr = res.as_mut_ptr() as *mut T;
                core::ptr::write(bytes_ptr, value);
                Ok(&mut *bytes_ptr)
            },
            Err(e) => { Err((e, value)) }
        }
        
    }

    // Allocate slice for an object of type T, copies all of it to that space, and returns a reference to that slice
    pub fn alloc_slice<'a, T>(&'a self, src: &[T]) -> Result<&'a mut [T], ArenaError>
    where
        T: Copy,
    {
        let byte_size = src.len() * core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        let allocated_bytes: &mut [u8] = self.alloc_align(byte_size, align)?;
        unsafe {
            let bytes_ptr: *mut T = allocated_bytes.as_mut_ptr() as *mut T;
            let allocated_slice = core::slice::from_raw_parts_mut(bytes_ptr, src.len());
            
            allocated_slice.copy_from_slice(src);
            Ok(allocated_slice)
        }
    }

}

// heap fallback
impl<const N: usize> Arena<N> {

    // Allocates memory on the heap in bytes, stores the pointer to the head of the arena, and returns the pointer to that memory
    pub unsafe fn alloc_heap_bytes(&self, size: usize, align: usize) -> Result<*mut u8, ArenaError> {

        unsafe {

            let head_layout = Layout::new::<HeapNode>();
            let data_layout = Layout::from_size_align(size, align).map_err(|_| ArenaError::AlignmentError)?;
            let (new_layout, new_offset) = head_layout.extend(data_layout).map_err(|_| ArenaError::AllocationError)?;

            let alloc_block_ptr: *mut u8 = alloc(new_layout);
            if alloc_block_ptr.is_null() { return Err(ArenaError::AllocationError); }
            let new_heapnode_ptr: *mut HeapNode = alloc_block_ptr as *mut HeapNode;
            
            self.update_heapnode_with_layout(new_heapnode_ptr, new_layout);
            Ok(alloc_block_ptr.add(new_offset))

        }

    }

    // Allocates memory on the heap for object T, and moves object into the heap memory, also returns value if failure
    pub fn alloc_heap<'a, T>(&'a self, value: T) -> Result<AllocSuccess<'a, T>, (ArenaError, T)> {

        unsafe {
            let size = core::mem::size_of::<T>();
            let align = core::mem::align_of::<T>();
            let allocation_res: Result<*mut u8, ArenaError>  = self.alloc_heap_bytes(size, align);
            match allocation_res {
                Ok(res) => {
                    let heap_ptr = res as *mut T;
                    core::ptr::write(heap_ptr, value);
                    Ok(AllocSuccess {
                        alloc_res: &mut *heap_ptr, 
                        alloc_loc: AllocLocation::HeapAlloc,
                    })
                },
                Err(e) => { Err((e, value)) }
            }
        }

    }

    // Allocates memory on the heap for a slice of object T, copies the slice and returns the reference
    pub fn alloc_heap_slice<'a, T>(&'a self, src: &[T]) -> Result<AllocSuccess<'a, [T]>, ArenaError>
    where
        T: Copy
    {

        unsafe {

            let byte_size = src.len() * core::mem::size_of::<T>();
            let align = core::mem::align_of::<T>();
            let heap_ptr = self.alloc_heap_bytes(byte_size, align)? as *mut T;
            let allocated_slice = &mut *core::ptr::slice_from_raw_parts_mut(heap_ptr, src.len());
            allocated_slice.copy_from_slice(src);
            Ok(AllocSuccess { 
                alloc_res: allocated_slice, 
                alloc_loc: AllocLocation::HeapAlloc, 
            })

        }

    }

    // Attempts to allocate memory in the arena for an object T, if fails, tries on the heap, if fails, returns the object
    pub fn try_alloc<'a, T>(&'a self, value: T) -> Result<AllocSuccess<'a, T>, (ArenaError, T)> {

        let arena_alloc_res: Result<&mut T, (ArenaError, T)> = self.alloc(value);
        match arena_alloc_res {
            Ok(res) => {
                Ok(AllocSuccess { 
                    alloc_res: res, 
                    alloc_loc: AllocLocation::ArenaAlloc, 
                })
            },
            Err(res) => {
                let heap_alloc_res: AllocSuccess<'a, T> = self.alloc_heap(res.1)?;
                Ok(heap_alloc_res)
            }
        }


    }

    // Attemptes to allocate memory in the arena for a slice of object T, if fails, tries on the heap, raises error if fails
    pub fn try_alloc_slice<'a, T: Copy>(&'a self, src: &[T]) -> Result<AllocSuccess<'a, [T]>, ArenaError>
    where
        T: Copy
    {

        match self.alloc_slice(src) {
            Ok(res) => {
                Ok(AllocSuccess { 
                    alloc_res: res, 
                    alloc_loc: AllocLocation::ArenaAlloc, 
                })
            },
            Err(_) => {
                Ok(self.alloc_heap_slice(src)?)
            }
        }

    }

}

// Resets arena and deallocates heap objects automatically on the arena going out of scope
impl<const N: usize> Drop for Arena<N> {

    fn drop(&mut self) {
        self.reset();
    }

}