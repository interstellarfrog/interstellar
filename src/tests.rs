use alloc::boxed::Box;
use alloc::vec::Vec;
use crate:: allocator::HEAP_SIZE;
use crate::print;

pub fn assert_eq_custom(x: u32, y: u32) {
    if x != y {
        print!("Failed");
        return;
    }
    print!("[OK]");
    return; 
}


pub fn main() {
    print!("\nSimple Addition...");
    assert_eq_custom(1 + 1, 1 + 1);

    print!("\nSimple Multiplication...");
    assert_eq_custom(5 * 5, 5 * 5);

    print!("\nSimple Division...");
    assert_eq_custom(50 / 5, 50 / 5);

    print!("\nSimple Allocation...");
    simple_allocation();

    print!("\nLarge Vector Allocation...");
    large_vec();

    print!("\nLarge Long Lived Box Allocation");
    many_boxes_long_lived();






}


//########################################
// In OS Tests
//########################################


fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    assert_eq_custom(*heap_value_1, 41);
}


fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq_custom(vec.iter().sum::<u32>(), (n - 1) * n / 2);
}

fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let _x = Box::new(i);
        if i == HEAP_SIZE / 3 || i == (HEAP_SIZE / 3) * 2  {
            print!(".");
        }
    }
    print!(".");
    assert_eq_custom(*long_lived, 1)
}