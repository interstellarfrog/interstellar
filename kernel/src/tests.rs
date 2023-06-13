//Copyright (C) <2023>  <interstellarfrog>
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use crate:: allocator::HEAP_SIZE;
use crate::print;

pub fn assert_eq_custom(x: u32, y: u32) {
    if x != y {
        print!("Failed");
        return;
    }
    print!("[OK]");
}


pub fn main() {
    print!("\nSimple Addition...");
    assert_eq_custom(1 + 1, 1 + 1);

    print!("\nSimple Subtraction...");
    assert_eq_custom(5 - 2, 3);

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

    print!("\nSimple Remainder...");
    simple_remainder();

    print!("\nVector Access...");
    vector_access();

    print!("\nVector Iteration...");
    vector_iteration();

    print!("\nBox Swap...");
    box_swap();

    print!("\nBox Clone...");
    box_clone();
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

fn simple_remainder() {
    assert_eq_custom(10 % 3, 1);
}

fn vector_access() {
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq_custom(vec[2], 3);
}

fn vector_iteration() {
    let vec = vec![1, 2, 3, 4, 5];
    let sum = vec.iter().sum::<u32>();
    assert_eq_custom(sum, 15);
}

fn box_swap() {
    let mut box1 = Box::new(42);
    let mut box2 = Box::new(24);
    core::mem::swap(&mut box1, &mut box2);
    assert_eq!(*box1, 24);
    assert_eq_custom(*box2, 42);
}

fn box_clone() {
    let box1 = Box::new(42);
    let box2 = box1.clone();
    assert_eq_custom(*box1, *box2);
}


