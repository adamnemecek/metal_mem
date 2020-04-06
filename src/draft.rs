// /// A draining iterator for `Vec<T>`.
// ///
// /// This `struct` is created by the [`drain`] method on [`Vec`].
// ///
// /// [`drain`]: struct.Vec.html#method.drain
// /// [`Vec`]: struct.Vec.html
// #[stable(feature = "drain", since = "1.6.0")]
// pub struct Drain<'a, T: 'a> {
//     /// Index of tail to preserve
//     tail_start: usize,
//     /// Length of tail
//     tail_len: usize,
//     /// Current remaining range to remove
//     iter: slice::Iter<'a, T>,
//     vec: NonNull<Vec<T>>,
// }

// // #[stable(feature = "collection_debug", since = "1.17.0")]
// impl<T: std::fmt::Debug> fmt::Debug for Drain<'_, T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
//     }
// }

// impl<'a, T> Drain<'a, T> {
//     /// Returns the remaining items of this iterator as a slice.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # #![feature(vec_drain_as_slice)]
//     /// let mut vec = vec!['a', 'b', 'c'];
//     /// let mut drain = vec.drain(..);
//     /// assert_eq!(drain.as_slice(), &['a', 'b', 'c']);
//     /// let _ = drain.next().unwrap();
//     /// assert_eq!(drain.as_slice(), &['b', 'c']);
//     /// ```
//     // #[unstable(feature = "vec_drain_as_slice", reason = "recently added", issue = "58957")]
//     pub fn as_slice(&self) -> &[T] {
//         self.iter.as_slice()
//     }
// }

// // #[stable(feature = "drain", since = "1.6.0")]
// unsafe impl<T: Sync> Sync for Drain<'_, T> {}
// // #[stable(feature = "drain", since = "1.6.0")]
// unsafe impl<T: Send> Send for Drain<'_, T> {}

// // #[stable(feature = "drain", since = "1.6.0")]
// impl<T> Iterator for Drain<'_, T> {
//     type Item = T;

//     #[inline]
//     fn next(&mut self) -> Option<T> {
//         self.iter.next().map(|elt| unsafe { ptr::read(elt as *const _) })
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// // #[stable(feature = "drain", since = "1.6.0")]
// impl<T> DoubleEndedIterator for Drain<'_, T> {
//     #[inline]
//     fn next_back(&mut self) -> Option<T> {
//         self.iter.next_back().map(|elt| unsafe { ptr::read(elt as *const _) })
//     }
// }

// // #[stable(feature = "drain", since = "1.6.0")]
// impl<T> Drop for Drain<'_, T> {
//     fn drop(&mut self) {
//         // exhaust self first
//         self.for_each(drop);

//         if self.tail_len > 0 {
//             unsafe {
//                 let source_vec = self.vec.as_mut();
//                 // memmove back untouched tail, update to new length
//                 let start = source_vec.len();
//                 let tail = self.tail_start;
//                 if tail != start {
//                     let src = source_vec.as_ptr().add(tail);
//                     let dst = source_vec.as_mut_ptr().add(start);
//                     ptr::copy(src, dst, self.tail_len);
//                 }
//                 source_vec.set_len(start + self.tail_len);
//             }
//         }
//     }
// }

// // #[stable(feature = "drain", since = "1.6.0")]
// impl<T> ExactSizeIterator for Drain<'_, T> {
//     fn is_empty(&self) -> bool {
//         self.iter.is_empty()
//     }
// }

// // #[unstable(feature = "trusted_len", issue = "37572")]
// unsafe impl<T> TrustedLen for Drain<'_, T> {}

// // #[stable(feature = "fused", since = "1.26.0")]
// impl<T> FusedIterator for Drain<'_, T> {}

// /// A splicing iterator for `Vec`.
// ///
// /// This struct is created by the [`splice()`] method on [`Vec`]. See its
// /// documentation for more.
// ///
// /// [`splice()`]: struct.Vec.html#method.splice
// /// [`Vec`]: struct.Vec.html
// #[derive(Debug)]
// // #[stable(feature = "vec_splice", since = "1.21.0")]
// pub struct Splice<'a, I: Iterator + 'a> {
//     drain: Drain<'a, I::Item>,
//     replace_with: I,
// }

// // #[stable(feature = "vec_splice", since = "1.21.0")]
// impl<I: Iterator> Iterator for Splice<'_, I> {
//     type Item = I::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.drain.next()
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.drain.size_hint()
//     }
// }

// // #[stable(feature = "vec_splice", since = "1.21.0")]
// impl<I: Iterator> DoubleEndedIterator for Splice<'_, I> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.drain.next_back()
//     }
// }

// // #[stable(feature = "vec_splice", since = "1.21.0")]
// impl<I: Iterator> ExactSizeIterator for Splice<'_, I> {}

// // #[stable(feature = "vec_splice", since = "1.21.0")]
// impl<I: Iterator> Drop for Splice<'_, I> {
//     fn drop(&mut self) {
//         self.drain.by_ref().for_each(drop);

//         unsafe {
//             if self.drain.tail_len == 0 {
//                 self.drain.vec.as_mut().extend(self.replace_with.by_ref());
//                 return;
//             }

//             // First fill the range left by drain().
//             if !self.drain.fill(&mut self.replace_with) {
//                 return;
//             }

//             // There may be more elements. Use the lower bound as an estimate.
//             // FIXME: Is the upper bound a better guess? Or something else?
//             let (lower_bound, _upper_bound) = self.replace_with.size_hint();
//             if lower_bound > 0 {
//                 self.drain.move_tail(lower_bound);
//                 if !self.drain.fill(&mut self.replace_with) {
//                     return;
//                 }
//             }

//             // Collect any remaining elements.
//             // This is a zero-length vector which does not allocate if `lower_bound` was exact.
//             let mut collected = self.replace_with.by_ref().collect::<Vec<I::Item>>().into_iter();
//             // Now we have an exact count.
//             if collected.len() > 0 {
//                 self.drain.move_tail(collected.len());
//                 let filled = self.drain.fill(&mut collected);
//                 debug_assert!(filled);
//                 debug_assert_eq!(collected.len(), 0);
//             }
//         }
//         // Let `Drain::drop` move the tail back if necessary and restore `vec.len`.
//     }
// }

// /// Private helper methods for `Splice::drop`
// impl<T> Drain<'_, T> {
//     /// The range from `self.vec.len` to `self.tail_start` contains elements
//     /// that have been moved out.
//     /// Fill that range as much as possible with new elements from the `replace_with` iterator.
//     /// Returns `true` if we filled the entire range. (`replace_with.next()` didn’t return `None`.)
//     unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
//         let vec = self.vec.as_mut();
//         let range_start = vec.len;
//         let range_end = self.tail_start;
//         let range_slice =
//             slice::from_raw_parts_mut(vec.as_mut_ptr().add(range_start), range_end - range_start);

//         for place in range_slice {
//             if let Some(new_item) = replace_with.next() {
//                 ptr::write(place, new_item);
//                 vec.len += 1;
//             } else {
//                 return false;
//             }
//         }
//         true
//     }

//     /// Makes room for inserting more elements before the tail.
//     unsafe fn move_tail(&mut self, extra_capacity: usize) {
//         let vec = self.vec.as_mut();
//         let used_capacity = self.tail_start + self.tail_len;
//         vec.buf.reserve(used_capacity, extra_capacity);

//         let new_tail_start = self.tail_start + extra_capacity;
//         let src = vec.as_ptr().add(self.tail_start);
//         let dst = vec.as_mut_ptr().add(new_tail_start);
//         std::ptr::copy(src, dst, self.tail_len);
//         self.tail_start = new_tail_start;
//     }
// }

// /// An iterator produced by calling `drain_filter` on Vec.
// #[unstable(feature = "drain_filter", reason = "recently added", issue = "43244")]
// #[derive(Debug)]
// pub struct DrainFilter<'a, T, F>
// where
//     F: FnMut(&mut T) -> bool,
// {
//     vec: &'a mut Vec<T>,
//     /// The index of the item that will be inspected by the next call to `next`.
//     idx: usize,
//     /// The number of items that have been drained (removed) thus far.
//     del: usize,
//     /// The original length of `vec` prior to draining.
//     old_len: usize,
//     /// The filter test predicate.
//     pred: F,
//     /// A flag that indicates a panic has occurred in the filter test prodicate.
//     /// This is used as a hint in the drop implmentation to prevent consumption
//     /// of the remainder of the `DrainFilter`. Any unprocessed items will be
//     /// backshifted in the `vec`, but no further items will be dropped or
//     /// tested by the filter predicate.
//     panic_flag: bool,
// }

// #[unstable(feature = "drain_filter", reason = "recently added", issue = "43244")]
// impl<T, F> Iterator for DrainFilter<'_, T, F>
// where
//     F: FnMut(&mut T) -> bool,
// {
//     type Item = T;

//     fn next(&mut self) -> Option<T> {
//         unsafe {
//             while self.idx < self.old_len {
//                 let i = self.idx;
//                 let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
//                 self.panic_flag = true;
//                 let drained = (self.pred)(&mut v[i]);
//                 self.panic_flag = false;
//                 // Update the index *after* the predicate is called. If the index
//                 // is updated prior and the predicate panics, the element at this
//                 // index would be leaked.
//                 self.idx += 1;
//                 if drained {
//                     self.del += 1;
//                     return Some(std::ptr::read(&v[i]));
//                 } else if self.del > 0 {
//                     let del = self.del;
//                     let src: *const T = &v[i];
//                     let dst: *mut T = &mut v[i - del];
//                     std::ptr::copy_nonoverlapping(src, dst, 1);
//                 }
//             }
//             None
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (0, Some(self.old_len - self.idx))
//     }
// }

// #[unstable(feature = "drain_filter", reason = "recently added", issue = "43244")]
// impl<T, F> Drop for DrainFilter<'_, T, F>
// where
//     F: FnMut(&mut T) -> bool,
// {
//     fn drop(&mut self) {
//         struct BackshiftOnDrop<'a, 'b, T, F>
//         where
//             F: FnMut(&mut T) -> bool,
//         {
//             drain: &'b mut DrainFilter<'a, T, F>,
//         }

//         impl<'a, 'b, T, F> Drop for BackshiftOnDrop<'a, 'b, T, F>
//         where
//             F: FnMut(&mut T) -> bool,
//         {
//             fn drop(&mut self) {
//                 unsafe {
//                     if self.drain.idx < self.drain.old_len && self.drain.del > 0 {
//                         // This is a pretty messed up state, and there isn't really an
//                         // obviously right thing to do. We don't want to keep trying
//                         // to execute `pred`, so we just backshift all the unprocessed
//                         // elements and tell the vec that they still exist. The backshift
//                         // is required to prevent a double-drop of the last successfully
//                         // drained item prior to a panic in the predicate.
//                         let ptr = self.drain.vec.as_mut_ptr();
//                         let src = ptr.add(self.drain.idx);
//                         let dst = src.sub(self.drain.del);
//                         let tail_len = self.drain.old_len - self.drain.idx;
//                         src.copy_to(dst, tail_len);
//                     }
//                     self.drain.vec.set_len(self.drain.old_len - self.drain.del);
//                 }
//             }
//         }

//         let backshift = BackshiftOnDrop { drain: self };

//         // Attempt to consume any remaining elements if the filter predicate
//         // has not yet panicked. We'll backshift any remaining elements
//         // whether we've already panicked or if the consumption here panics.
//         if !backshift.drain.panic_flag {
//             backshift.drain.for_each(drop);
//         }
//     }
// }