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

use frunk::hlist::{HCons, HNil, Selector};

pub trait Projector<Targets, Indicies> {
    type Projection<'a>
    where
        Self: 'a,
        Targets: 'a;

    fn project(&self) -> Self::Projection<'_>;
}

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

    // // This will intentionally not compile
    // #[test]
    // fn project_non_existant() {
    //     let h = hlist![1u32, "hello world", 42i64, true];
    //     type S = HList![i64, f32]; // <--- There is not 'f32' in our example HList

    //     let projection = h.project_ref_ext::<S, _>(); // <--- therefore this method will not work
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

    #[test]
    fn massive_test() {
        let h = hlist![1u32, 12f64, 99i64, String::from("hello world"), true];
        type S = HList![String, bool, i64, u32, f64];

        let projection = h.project_ref_ext::<S, _>();
        let hlist_pat![string, bool, int64, un32, float64] = projection;

        assert_eq!(*string, "hello world".to_string());
        assert_eq!(*bool, true);
        assert_eq!(*int64, 99i64);
        assert_eq!(*un32, 1u32);
        assert_eq!(*float64, 12f64);
    }
}
