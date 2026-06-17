// Project<for all T in S> => [&T iff T in H]
// 'iff' because frunk's Selector method doesn't exist if T isn't in H
// therefore my Project method doesn't exist if there exists T in S s.t. T not in H
// and by 'exists' I mean the compiler is able to prove coherence.
//
// in other words:
// ProjectRef<'a, S, H> exists <=> S is a proper subset of H
//
// Like how a Sculptor is a Plucker on steroids,
// Projector is like a Selector on steroids.

use frunk::{
    hlist::{HCons, HNil, Selector},
    indices::{Here, There},
};

// Projection will recurse over the target type list S,
// searching for its corresponding type in H,
// then use Selector to extract a ref
pub trait Projector<'a, S, Indicies> {
    type Output;
    fn project_ref(&'a self) -> Self::Output;
}

// base case: target has been emptied
impl<'a, Source, Idx> Projector<'a, HNil, Idx> for Source
where
    Source: Contains<HNil, Idx>, // as it turns out, Contains is only needed for the base case for some reason.
{
    type Output = HNil;
    fn project_ref(&'a self) -> Self::Output {
        HNil
    }
}

// inductive case: call selector on head and recurse through targets
impl<'a, THead, TTail, SHead, STail, IdxH, IdxT>
    Projector<'a, HCons<THead, TTail>, HCons<IdxH, IdxT>> for HCons<SHead, STail>
where
    HCons<SHead, STail>: Selector<THead, IdxH> + Projector<'a, TTail, IdxT>, // + Contains<THead, IdxH>,
    THead: 'a,
{
    type Output = HCons<&'a THead, <HCons<SHead, STail> as Projector<'a, TTail, IdxT>>::Output>;

    fn project_ref(&'a self) -> Self::Output {
        HCons {
            head: self.get(),
            tail: <HCons<SHead, STail> as Projector<'a, TTail, IdxT>>::project_ref(&self),
        }
    }
}

// helper trait: moves generics to the function for usage ergonomics
// though, if this gets merged, can be replaced with a free fn implementation on HCons
pub trait ProjectRefExt {
    fn project_ref_ext<'a, S, Idx>(&'a self) -> <Self as Projector<'a, S, Idx>>::Output
    where
        Self: Projector<'a, S, Idx>;
}

impl<T> ProjectRefExt for T {
    fn project_ref_ext<'a, S, Idx>(&'a self) -> <Self as Projector<'a, S, Idx>>::Output
    where
        Self: Projector<'a, S, Idx>,
    {
        <Self as Projector<'a, S, Idx>>::project_ref(&self)
    }
}

// rust actually struggles to infer the pair-wise index hlist of our target types during compilation,
// so this is a helper trait that says "this type T is in HCons at this Index"
//
// Why are there two indexes?
// The generic is for recursive purposes and to prevent conflicting implementations.
// The associated type stores this generic index as data which can be used by the compiler.
//
// (This was discovered later after I made everything work)
// I was messing around and found out that everything still worked when I accidentally removed the Contains bound on the recursive step.
// Then I removed it from the base case and everything broke.
// So now I've come to realise that I know nothing about type inference.
// Was Rust already perfectly capable of inferring the index of each type, except for the HNil type?
// Did I waste 4 hours spread across 2 days of my life for nothing?
pub trait Contains<T, Idx> {
    type Index;
}

// Should a HNil contain itself?
// Do all types contain themselves?
// Its quite a philosophical question, but for the sake of pleasing the compiler I declare that is does.
impl Contains<HNil, Here> for HNil {
    type Index = Here;
}

impl<T, Tail> Contains<T, Here> for HCons<T, Tail> {
    type Index = Here;
}

impl<T, Head, Tail, Idx> Contains<T, There<Idx>> for HCons<Head, Tail>
where
    Tail: Contains<T, Idx>,
{
    type Index = There<Tail::Index>;
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    use frunk::{HList, hlist, hlist_pat};

    #[test]
    fn project_simple_type() {
        let h = hlist![1u32, "hello world", true];
        type S = HList![u32];

        let projection = h.project_ref_ext::<S, _>();
        let hlist_pat![value_ref] = projection;

        assert_eq!(*value_ref, 1u32);
    }

    #[test]
    fn project_multiple_types() {
        let h = hlist![1u32, "hello world", 42i64, true];
        type S = HList![u32, i64];

        let projection = h.project_ref_ext::<S, _>();
        let hlist_pat![usize_ref, isize_ref] = projection;

        assert_eq!(*usize_ref, 1u32);
        assert_eq!(*isize_ref, 42i64);
    }

    #[test]
    fn project_different_order() {
        // this test also checks if target order is preserved
        let h = hlist![1u32, "hello world", 42i64, true];
        type S = HList![i64, u32]; // i64 and u32 appear in differing orders here than in H

        let projection = h.project_ref_ext::<S, _>();
        let hlist_pat![isize_ref, usize_ref] = projection;

        assert_eq!(*isize_ref, 42i64);
        assert_eq!(*usize_ref, 1u32);
    }

    // This will intentionally not compile
    // #[test]
    // fn project_non_existant() {
    //     let h = hlist![1u32, "hello world", 42i64, true];
    //     type S = HList![i64, f32]; <--- There is not 'f32' in our example HList

    //     let projection = h.project_ref_ext::<S, _>(); <--- therefore this method will not work
    //     let hlist_pat![isize_ref, float_ref] = projection;

    //     assert_eq!(*isize_ref, 42i64);
    //     assert_eq!(*float_ref, 8f32);
    // }

    #[test]
    fn borrowed_values_are_refs() {
        let h = hlist![String::from("hello world")];
        type S = HList![String];

        let _projection = h.project_ref_ext::<S, _>();

        // h is not moved by projection
        h.prepend(true);
    }
}
