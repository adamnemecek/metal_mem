#[macro_export]
macro_rules! gpuvec {
    () => (
        $crate::GPUVec::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::from_elem($elem, $n)
    );
    ($($x:expr),+ $(,)?) => (
        GPUVec::from_slice1(&<[_]>::into_vec(Box::new( [$($x),+])))
        // GPUVec::from_slice(&[$($x),+]])
    );
}