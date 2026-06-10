const DEFAULT_CAPACITY: usize = 1024;

#[derive(Debug)]
enum ArenaError {
    AllocationError,
    AlignmentError,
}

#[repr(align(8))]
pub struct Arena<const N: usize = DEFAULT_CAPACITY> {
    container: [u8; N],
    offset: usize,
}

// constructors and accessors
impl<const N: usize> Arena<N> {

    pub fn new() -> Self {

        Self {
            container: [0; N],
            offset: 0,
        } 

    }

    pub fn capacity(&self) -> usize { N }
    pub fn allocated_bytes(&self) -> usize { self.offset }

}

// direct allocation
impl<const N: usize> Arena<N> {

    pub fn alloc_bytes<'a>(&'a mut self, size: usize) -> Result<&'a mut [u8], ArenaError> {
        unsafe {
            if (self.offset + size) <= N {
                let start_ptr: *mut u8 = self.container.as_mut_ptr();
                self.offset += size;
                Ok(core::slice::from_raw_parts_mut(start_ptr.add(self.offset - size), size))
            } else {
                Err(ArenaError::AllocationError)
            }
        }
    }

    pub fn alloc_align<'a>(&'a mut self, size: usize, align: usize) -> Result<&'a mut [u8], ArenaError> {

        if align == 0 || (align & (align - 1)) != 0 { return Err(ArenaError::AlignmentError);}

        let cur_addr = unsafe { self.container.as_mut_ptr().add(self.offset) } as usize;
        let align_addr = (cur_addr + align - 1) & !(align - 1);
        let padding = align_addr - cur_addr;
        
        if self.offset + padding + size > N { return Err(ArenaError::AllocationError); }
        self.offset += padding;
        self.alloc_bytes(size)

    }

    pub fn reset(&mut self) -> () {
        self.offset = 0;
    }

}

// slice allocation
impl<const N: usize> Arena<N> {

    pub fn alloc_slice_bytes<'a>(&'a mut self, src: &[u8]) -> Result<&'a mut [u8], ArenaError> {
        let allocated_mem: &mut [u8] = self.alloc_bytes(src.len())?;
        allocated_mem.copy_from_slice(src);
        Ok(allocated_mem)
    }

    pub fn alloc_str<'a>(&'a mut self, slice: &str) -> Result<&'a mut str, ArenaError> {
        let allocated_str_bytes = self.alloc_slice_bytes(slice.as_bytes())?;
        Ok(core::str::from_utf8_mut(allocated_str_bytes).unwrap())
    }

}

// generic allocation
impl <const N: usize> Arena<N> {

    pub fn alloc<'a, T>(&'a mut self, value: T) -> Result<&'a mut T, ArenaError> {
        let size = core::mem::size_of::<T>();
        let allocated_bytes: &mut [u8]  = self.alloc_align(size, core::mem::align_of::<T>())?;
        unsafe { 
            let bytes_ptr = allocated_bytes.as_mut_ptr() as *mut T;
            core::ptr::write(bytes_ptr, value);
            Ok(&mut *bytes_ptr)
        }
    }

    pub fn alloc_slice<'a, T>(&'a mut self, src: &[T]) -> Result<&'a mut [T], ArenaError>
    where
        T: Copy,
    {

    }

}