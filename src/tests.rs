#[cfg(test)]

use super::arena::*;

#[test]
pub fn test_basic_allocation() {
    let mut arena = Arena::<1024>::new();
    assert_eq!(arena.capacity(), 1024);
    assert_eq!(arena.allocated_bytes(), 0);

    {
        let num1 = arena.alloc(42u32).unwrap();
        assert_eq!(*num1, 42);
    }

    {
      let num2 = arena.alloc(100u64).unwrap();
        assert_eq!(*num2, 100);  
    }

    let text = arena.alloc_str("Hewwo :3").unwrap();
    assert_eq!(text, "Hewwo :3");
}

#[test]
pub fn test_slice_allocation() {
    let mut arena = Arena::<512>::new();
    let source_data = [10, 20, 30, 40];
    
    let allocated_slice = arena.alloc_slice(&source_data).unwrap();
    assert_eq!(allocated_slice, &source_data);
    
    allocated_slice[0] = 99;
    assert_ne!(allocated_slice[0], source_data[0]);
}

#[test]
pub fn test_reset_and_overwrite() {
    let mut arena = Arena::<32>::new();
    
    let _ = arena.alloc_slice(&[1, 2, 3, 4]).unwrap();
    let bytes_used_before = arena.allocated_bytes();
    assert!(bytes_used_before > 0);

    arena.reset();
    assert_eq!(arena.allocated_bytes(), 0);

    let overwritten = arena.alloc_slice(&[5, 6, 7, 8]).unwrap();
    assert_eq!(overwritten, &[5, 6, 7, 8]);
}

#[test]
pub fn test_allocation_failure() {

    let mut arena = Arena::<4>::new();
    assert!(arena.alloc(1u32).is_ok()); 
    let overflow_attempt = arena.alloc(42u32);
    assert!(matches!(overflow_attempt, Err(ArenaError::AllocationError)));
}

#[test]
pub fn test_alignment() {
    let mut arena = Arena::<2048>::new();

    let byte_ref = arena.alloc(1u8).unwrap();
    let byte_addr = byte_ref as *const u8 as usize;
    println!("u8 address:  {:#x} (Alignment: 1)", byte_addr);

    let u32_ref = arena.alloc(100u32).unwrap();
    let u32_addr = u32_ref as *const u32 as usize;
    println!("u32 address: {:#x} (Alignment: 4)", u32_addr);

    let u64_ref = arena.alloc(5000u64).unwrap();
    let u64_addr = u64_ref as *const u64 as usize;
    println!("u64 address: {:#x} (Alignment: 8)", u64_addr);

    assert_eq!(u32_addr % 4, 0, "u32 address is not aligned to 4 :/");
    assert_eq!(u64_addr % 8, 0, "u64 address is not aligned to 8 :/");
    
    assert_ne!(byte_addr, u32_addr);
}
