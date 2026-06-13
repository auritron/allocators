use std::alloc::alloc;

#[cfg(test)]

use super::arena::*;

#[test]
pub fn test_byte_allocation () {
    let arena = Arena::<256>::new();
    let alloc_1 = arena.alloc_bytes(64); //64B
    assert!(alloc_1.is_ok());
    let alloc_2 = arena.alloc(67u128).unwrap(); //80B
    assert_eq!(*alloc_2, 67u128);
    let alloc_3 = arena.alloc(1000000u64).unwrap(); //88B
    *alloc_3 = 2000000;
    assert_eq!(*alloc_3, 2000000u64);
    let alloc_4 = arena.alloc_bytes(168); //256B
    assert!(alloc_4.is_ok());
    let alloc_5 = arena.alloc(64); //FAIL
    assert!(alloc_5.is_err());
}

#[test]
pub fn test_capacity_and_offset() {
    let arena = Arena::<128>::new();
    assert_eq!(arena.capacity(), 128);
    assert_eq!(arena.allocated_bytes(), 0);
    let alloc_1 = arena.alloc_bytes(37).unwrap();
    assert_eq!(arena.capacity(), 128);
    assert_eq!(arena.allocated_bytes(), 37);
    let alloc_2 = arena.alloc_bytes(83).unwrap();
    assert_eq!(arena.capacity(), 128);
    assert_eq!(arena.allocated_bytes(), 120);
    let alloc_3 = arena.alloc_bytes(55);
    assert!(alloc_3.is_err());
    assert_eq!(arena.capacity(), 128);
    assert_eq!(arena.allocated_bytes(), 120);
}

#[test]
pub fn test_slice_allocation() {
    {
        let arena = Arena::<64>::new();
        let arr_1 = [10, 20, 30, 40, 50] as [u8; 5]; //5B
        let slice_1 = &arr_1[1..4];
        let alloc_1 = arena.alloc_slice_bytes(slice_1);
        assert!(alloc_1.is_ok());
        assert_eq!(arena.allocated_bytes(), 3);
        let vec_2: Vec<u8> = (1..=127).collect();
        let slice_2 = &vec_2 as &[u8];
        let alloc_2 = arena.alloc_slice_bytes(slice_2);
        assert!(alloc_2.is_err());
        assert_eq!(arena.allocated_bytes(), 3);
    }
    {
        let arena = Arena::<64>::new();
        let str_3 = "Heyyyy, what's up :D";
        let alloc_3 = arena.alloc_str(str_3);
        assert!(alloc_3.is_ok());
        assert_eq!(arena.allocated_bytes(), 20);
        assert_eq!(alloc_3.unwrap().chars().nth(6), Some(','));
        let str_4 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let alloc_4 = arena.alloc_str(str_4);
        assert!(alloc_4.is_ok());
        assert_eq!(arena.allocated_bytes(), 64);
        let str_5 = "This will fail.";
        let alloc_5 = arena.alloc_str(str_5);
        assert!(alloc_5.is_err());
        assert_eq!(arena.allocated_bytes(), 64);
    }
    {
        let arena = Arena::<256>::new();
        let vec_6: Vec<u32> = (0..=31).collect();
        let slice_6 = &vec_6 as &[u32];
        let alloc_6 = arena.alloc_slice(slice_6); //128B
        assert!(alloc_6.is_ok());
        let slice_6_comp = &vec_6 as &[u32];
        let s6_comp_1 = alloc_6.unwrap();
        assert_eq!(s6_comp_1, slice_6_comp);
        let arr_7 = [true; 32];
        let slice_7 = &arr_7;
        let alloc_7 = arena.alloc_slice(slice_7);
        assert_eq!(s6_comp_1[6], 6);
        assert!(alloc_7.is_ok());
        assert!(alloc_7.unwrap()[14]);
        let str_8 = "nya~";
        let alloc_8 = arena.alloc_str(str_8);
        assert_eq!(alloc_8.unwrap(), "nya~");
        assert_eq!(arena.allocated_bytes(), 164);
    }
}

#[test]
pub fn test_manual_reset() {
    let mut arena = Arena::<1024>::new();
    let slice_9 = &[0u128; 64];
    let alloc_9 = arena.alloc_slice(slice_9);
    assert!(alloc_9.is_ok());
    assert_eq!(arena.allocated_bytes(), 1024);
    arena.reset();
    let slice_10 = &[15u16; 37];
    let alloc_10 = arena.alloc_slice(slice_10);
    assert!(alloc_10.is_ok());
    assert_eq!(arena.allocated_bytes(), 74);
}

#[test]
pub fn test_allocation_alignment() {
    let arena = Arena::<2048>::new();

    let alloc_10 = arena.alloc(1u8).unwrap();
    let byte_addr = alloc_10 as *const u8 as usize;
    println!("u8 address:  {:#x} (Alignment: 1)", byte_addr);

    let alloc_11 = arena.alloc(100u32).unwrap();
    let u32_addr = alloc_11 as *const u32 as usize;
    println!("u32 address: {:#x} (Alignment: 4)", u32_addr);

    let alloc_12 = arena.alloc(5000u64).unwrap();
    let u64_addr = alloc_12 as *const u64 as usize;
    println!("u64 address: {:#x} (Alignment: 8)", u64_addr);

    assert_eq!(u32_addr % 4, 0, "u32 address is not aligned to 4 :/");
    assert_eq!(u64_addr % 8, 0, "u64 address is not aligned to 8 :/");
    
    assert_ne!(byte_addr, u32_addr);
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct localObject { // Size: 256B, Align: 16B
    field_1: [u128; 15],
    field_2: u16,
    field_3: [bool; 4],
}

#[test]
pub fn test_manual_heap_allocation() {
    let arena = Arena::<1024>::new();

    let alloc_13 = unsafe { arena.alloc_heap_bytes(64, 8) };
    assert!(alloc_13.is_ok());
    
    let obj = localObject {
        field_1: [0u128; 15],
        field_2: 42u16,
        field_3: [true, false, true, false],
    };
    let alloc_14 = arena.alloc_heap(obj);
    assert!(alloc_14.is_ok());
    let success_obj = alloc_14.unwrap();
    assert_eq!(success_obj.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(success_obj.alloc_res.field_2, 42u16);

    let src_slice = [obj; 2];
    let alloc_15 = arena.alloc_heap_slice(&src_slice);
    assert!(alloc_15.is_ok());
    let success_slice = alloc_15.unwrap();
    assert_eq!(success_slice.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(success_slice.alloc_res.len(), 2);
    assert_eq!(success_slice.alloc_res[0].field_2, 42u16);

    unsafe {
        let alloc_16 = arena.alloc_heap_bytes(usize::MAX - 2, 8);
        assert!(alloc_16.is_err());
    }

    let alloc_17 = arena.alloc_heap_slice(&[obj; 2]);
    assert!(alloc_17.is_ok());
}

#[test]
pub fn test_attempted_heap_allocation() {
    let arena = Arena::<480>::new();
    let obj = localObject {
        field_1: [1u128; 15],
        field_2: 100u16,
        field_3: [false; 4],
    };
    let alloc_18 = arena.try_alloc(obj).unwrap();
    assert_eq!(alloc_18.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(alloc_18.alloc_res.field_2, 100u16);
    assert_eq!(arena.allocated_bytes(), 256);

    let alloc_19 = arena.try_alloc(obj).unwrap();
    assert_eq!(alloc_19.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(alloc_19.alloc_res.field_2, 100u16);
    assert_eq!(arena.allocated_bytes(), 256);

    let arena_slice = Arena::<1000>::new();
    let src_slice = [obj; 2];

    let alloc_20 = arena_slice.try_alloc_slice(&src_slice).unwrap();
    assert_eq!(alloc_20.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena_slice.allocated_bytes(), 512);

    let alloc_21 = arena_slice.try_alloc_slice(&src_slice).unwrap();
    assert_eq!(alloc_21.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(arena_slice.allocated_bytes(), 512);
}

#[test]
pub fn test_hybrid_allocation() {
    let mut arena = Arena::<64>::new();

    let alloc_22 = arena.alloc_bytes(16).unwrap();
    assert_eq!(arena.allocated_bytes(), 16);

    let val_16b = [0u64; 2];
    let alloc_23 = arena.try_alloc(val_16b).unwrap();
    assert_eq!(alloc_23.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena.allocated_bytes(), 32);

    let alloc_24 = arena.try_alloc(42u64).unwrap();
    assert_eq!(alloc_24.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena.allocated_bytes(), 40);

    alloc_22[5] = 255;
    assert_eq!(alloc_22[5], 255);

    let alloc_25 = arena.alloc_slice(&[0u8; 32]);
    assert!(alloc_25.is_err());
    assert_eq!(arena.allocated_bytes(), 40);

    let large_val = [0u64; 8];
    let alloc_26 = arena.try_alloc(large_val).unwrap();
    assert_eq!(alloc_26.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(arena.allocated_bytes(), 40);

    let alloc_27 = arena.alloc_heap(val_16b).unwrap();
    assert_eq!(alloc_27.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(arena.allocated_bytes(), 40);

    let alloc_28 = arena.try_alloc(val_16b).unwrap();
    assert_eq!(alloc_28.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena.allocated_bytes(), 56);

    let alloc_29 = arena.try_alloc(100u64).unwrap();
    assert_eq!(alloc_29.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena.allocated_bytes(), 64);

    let alloc_30 = arena.try_alloc([0u8; 32]).unwrap();
    assert_eq!(alloc_30.alloc_loc, AllocLocation::HeapAlloc);
    assert_eq!(arena.allocated_bytes(), 64);

    arena.reset();
    assert_eq!(arena.allocated_bytes(), 0);

    let alloc_31 = arena.try_alloc([0u64; 8]).unwrap();
    assert_eq!(alloc_31.alloc_loc, AllocLocation::ArenaAlloc);
    assert_eq!(arena.allocated_bytes(), 64);
}