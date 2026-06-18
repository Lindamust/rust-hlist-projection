usage example:
```rust
type S = HList![String, bool, i64, u32, f64];

let projection = h.project_ref_ext::<S, _>();
let hlist_pat![string, bool, int64, un32, float64] = projection;

assert_eq!(*string, &"hello world".to_string());
assert_eq!(bool, &true);
assert_eq!(int64, &99i64);
assert_eq!(un32, &1u32);
assert_eq!(float64, &12f64);
```
