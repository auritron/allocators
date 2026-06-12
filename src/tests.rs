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

}

#[test]
pub fn test_allocation_alignment() {

}

#[test]
pub fn test_manual_heap_allocation() {

}

#[test]
pub fn test_attempted_heap_allocation() {

}

#[test]
pub fn test_real_world_allocation() {

}