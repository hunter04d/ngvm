#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

pub mod primitives;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
