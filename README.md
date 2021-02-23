# on_demand

This Rust crate provides a macro to generate aux macros for on-demand usage.

```toml
on_demand = "0.1"
```

## Examples

```rust
use on_demand::generate_on_demand_macro;

fn foo() {
    generate_on_demand_macro!(a: usize = None, {
        println!("a");
        1
    });
    generate_on_demand_macro!(b: usize = None, {
        println!("b");
        let a_data = on_demand_get_a!();
        2 + *a_data
    });
    generate_on_demand_macro!(c: usize = None, {
        println!("c");
        let a_data = on_demand_get_a!();
        let b_data = on_demand_get_b!();
        3 + *a_data + *b_data
    });

    let c_data = on_demand_get_c!();
    assert_eq!(*c_data, 6);
}
```

After calling `generate_on_demand_macro` to the variable (for example, `a`), three new macros `on_demand_get_a`, `on_demand_get_a_mut` and `on_demand_into_a` are generated. When `on_demand_get_a` is called, it determines whether `a` has been calculated. If it is, then returns the reference to its data, otherwise calls the expression given as the second parameter of `generate_on_demand_macro`, then assigns the returned value to `a`. The other two generated macros do the similar job, but returns the mutable reference to, or takes ownership of `a`.

In all, this means calculating `a` lazily using the expression.
