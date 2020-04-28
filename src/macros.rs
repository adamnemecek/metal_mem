
#[macro_export]
macro_rules! gpuvec {
    () => (
        $crate::GPUVec::new()
    );
    // ($elem:expr; $n:expr) => (
    //     $crate::GPUVec::from_elem($elem, $n)
    // );
    ($($x:expr),+ $(,)?) => (
        $crate::GPUVec::from_slice1(&<[_]>::into_vec(Box::new( [$($x),+])))
        // GPUVec::from_slice(&[$($x),+]])
    );
}

#[macro_export]
macro_rules! gpuvar {
    () => (
        $crate::GPUVar::new()
    );
    // ($elem:expr; $n:expr) => (
    //     $crate::GPUVar::with_value1($elem, $n)
    // );
    ($elem:expr) => (
        $crate::GPUVar::with_value1($elem)
    );
}

