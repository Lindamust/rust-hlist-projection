// Project<for all T in S> => &T iff T in H
// 'iff' because frunk's Selector method doesn't exist if T isn't in H
// therefore my Project method doesn't exist if there exists T in S s.t. T not in H
// and by 'exists' i mean the compiler is able to prove coherence (existance is dual to compiler proof)
//
// in other words:
// ProjectRef<'a, S, H> exists ⇔ S ⊆ H

use frunk::hlist::{HCons, HNil, Selector};
// HNil: zero-sized, end of a hlist
// HCons<H, T>: hlist node (head, tail) -> ...
// Selector<T, Idx>: pulls a reference to a type T iff T is in the HList in question (idx is a type witness)

pub trait ProjectRef<'a, S, Idx> {
    type Output;
    fn project_ref(&'a self) -> Self::Output;
}

// base case: target has been emptied
impl<'a, Source, Idx> ProjectRef<'a, HNil, Idx> for Source {
    type Output = HNil;
    fn project_ref(&'a self) -> Self::Output {
        HNil
    }
}

// inductive case: call selector on head and recurse through targets
impl<'a, THead, TTail, SHead, STail, IdxH, IdxT>
    ProjectRef<'a, HCons<THead, TTail>, HCons<IdxH, IdxT>> for HCons<SHead, STail>
where
    HCons<SHead, STail>: Selector<THead, IdxH> + ProjectRef<'a, TTail, IdxT>, // <-- here we assume that ProjectRef holds for the tail
    THead: 'a,
{
    type Output = HCons<&'a THead, <HCons<SHead, STail> as ProjectRef<'a, TTail, IdxT>>::Output>;

    fn project_ref(&'a self) -> Self::Output {
        HCons {
            head: self.get(),
            tail: <HCons<SHead, STail> as ProjectRef<'a, TTail, IdxT>>::project_ref(&self),
        }
    }
}

// helper trait: moves generics to the function for usage ergonomics
pub trait ProjectRefExt {
    fn project_ref_ext<'a, S, Idx>(&'a self) -> <Self as ProjectRef<'a, S, Idx>>::Output
    where
        Self: ProjectRef<'a, S, Idx>;
}

impl<T> ProjectRefExt for T {
    fn project_ref_ext<'a, S, Idx>(&'a self) -> <Self as ProjectRef<'a, S, Idx>>::Output
    where
        Self: ProjectRef<'a, S, Idx>,
    {
        <Self as ProjectRef<'a, S, Idx>>::project_ref(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frunk::{
        HList, hlist, hlist_pat,
        indices::{Here, There},
    };

    #[test]
    fn project_simple_type() {
        let h = hlist![1u32, "hello world", true];
        type S = HList![u32];

        let projection = h.project_ref_ext::<S, HCons<Here, HNil>>();
        let hlist_pat![value_ref] = projection;

        assert_eq!(*value_ref, 1u32);
    }

    #[test]
    fn project_multiple_types() {
        let h = hlist![1u32, "hello world", 42i64, true];
        type S = HList![u32, i64];

        let projection = h.project_ref_ext::<S, HCons<Here, HCons<There<There<Here>>, HNil>>>();
        let hlist_pat![usize_ref, isize_ref] = projection;
        assert_eq!(*usize_ref, 1u32);
        assert_eq!(*isize_ref, 42i64);
    }
}
