global MAX_DEPTH: u32 = 10;

unconstrained fn compute_triangular_number(n: u32) -> Field {
    let n_field = n as Field;
    n_field * (n_field + 1) / 2
}

fn main(input: Field, depth: u32) {
    assert(depth > 0);
    assert(depth <= MAX_DEPTH);
    // safety: we're calling an unconstrained function with a validated input
    // the depth parameter is already verified to be within safe bounds
    let expected_triangular = unsafe { compute_triangular_number(depth) };
    assert(input == expected_triangular);
    let depth_field = depth as Field;
    let doubled_input = input * 2;
    let triangular_formula = depth_field * (depth_field + 1);
    assert(doubled_input == triangular_formula);
}

#[test]
fn test_triangular_numbers() {
    main(1, 1);   
    main(3, 2);   
    main(6, 3);   
    main(10, 4);  
    main(15, 5);  
    main(21, 6);  
    main(28, 7);  
    main(36, 8);  
    main(45, 9);  
    main(55, 10); 
}

#[test(should_fail_with = "assertion failed")]
fn test_invalid_triangular() {
    main(14, 5); 
}

#[test(should_fail_with = "assertion failed")]
fn test_depth_too_large() {
    main(66, 11); 
}

#[test(should_fail_with = "assertion failed")]
fn test_depth_zero() {
    main(0, 0); 
}
