//! Wait for the first of several futures to complete.

// 3
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Result for [`select`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 10
pub enum Either<A, B> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
}

/// Wait for one of two futures to complete.
///
/// This function returns a new future which polls all the futures.
/// When one of them completes, it will complete with its result value.
///
/// The other future is dropped.
// 35
pub fn select<A, B>(a: A, b: B) -> Select<A, B>
where
    A: Future,
    B: Future,
{
    Select { a, b }
}

/// Future for the [`select`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
// 46
pub struct Select<A, B> {
    a: A,
    b: B,
}

// 53
impl<A, B> Future for Select<A, B>
where
    A: Future,
    B: Future,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either::Second(x));
        }
        Poll::Pending
    }
}
