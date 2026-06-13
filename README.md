# allocators
Custom allocators built using Rust :3
Just some personal low-level programming practice!

## Finished -
> Arena Allocator - Stack-allocatead, Heap-fallback, POD-safe, Thread-unsafe  
**[NOTE: The allocator is safe for POD data, but complex heap types like Strings and Vecs should NOT be allocated onto the heap, as deallocation can cause memory leaks.]**
