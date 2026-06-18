# About
HList projection is an operation that views a heterogeneous list through a type-level “lens” of sorts, producing a new HList of shared references to selected elements

“Projection” here means selecting and re-expressing specific components of a structure without moving or removing them, but instead borrowing them from the original.

### Exposes a trait
```rust
pub trait Projector<Targets, Indicies> {
    type Projection<'a>
    where
        Self: 'a,
        Targets: 'a;

    fn project(&self) -> Self::Projection<'_>;
}
```
### ...And a helper trait
```rust
// helper trait: moves generics to the function for usage ergonomics
// though, if this gets merged, can be replaced with a free fn implementation on HCons
pub trait ProjectRefExt {
    fn project_ref_ext<S, Idx>(&self) -> <Self as Projector<S, Idx>>::Projection<'_>
    where
        Self: Projector<S, Idx>;
}

impl<T> ProjectRefExt for T {
    fn project_ref_ext<S, Idx>(&self) -> <Self as Projector<S, Idx>>::Projection<'_>
    where
        Self: Projector<S, Idx>,
    {
        <Self as Projector<S, Idx>>::project(&self)
    }
}
```

# How it works
The trait works by recursing through the target types and calling `Selector` on the current head.
```rust 
// base case: target has been emptied
impl<Source> Projector<HNil, HNil> for Source {
    type Projection<'a>
        = HNil
    where
        Source: 'a,
        HNil: 'a;

    fn project(&self) -> Self::Projection<'_> {
        HNil
    }
}

// inductive case: call selector on target head and recurse through target tail
impl<THead, TTail, SHead, STail, IdxH, IdxT> Projector<HCons<THead, TTail>, HCons<IdxH, IdxT>>
    for HCons<SHead, STail>
where
    HCons<SHead, STail>: Selector<THead, IdxH> + Projector<TTail, IdxT>,
{
    type Projection<'a>
        = HCons<&'a THead, <HCons<SHead, STail> as Projector<TTail, IdxT>>::Projection<'a>>
    where
        HCons<SHead, STail>: 'a,
        HCons<THead, TTail>: 'a;

    fn project(&self) -> Self::Projection<'_> {
        HCons {
            head: self.get(),
            tail: <HCons<SHead, STail> as Projector<TTail, IdxT>>::project(&self),
        }
    }
}
```

# Usage Example
```rust
let h = hlist![1u32, 12f64, 99i64, String::from("hello world"), true];
type S = HList![String, bool, i64, u32, f64];

let projection = h.project_ref_ext::<S, _>();
let hlist_pat![string, bool, int64, un32, float64] = projection;

assert_eq!(*string, &"hello world".to_string());
assert_eq!(bool, &true);
assert_eq!(int64, &99i64);
assert_eq!(un32, &1u32);
assert_eq!(float64, &12f64);
```

# Failure Example
``` rust
// This will intentionally not compile
let h = hlist![1u32, "hello world", 42i64, true];
type S = HList![i64, f32]; // <--- There is not 'f32' in our example HList

let projection = h.project_ref_ext::<S, _>(); // <--- therefore this method will not work
let hlist_pat![isize_ref, float_ref] = projection;

assert_eq!(*isize_ref, 42i64);
assert_eq!(*float_ref, 8f32);
```

# Limitations
Currently only supports projections of shared references.

Implementing projections of mutable references would require require making multiple mutable references to the Source HList due to the recursive nature of the implementation thus far. Therefore, if mutable projection were to be implemented it would be through unsafe raw pointer splitting since the indices structurally guarantee non-overlapping memory positions.

Though, if possible, a workaround allowing for ~pseudo~ mutable projections could be to Sculpt the needed types, mutate them, then re-merge with the original remainder.
