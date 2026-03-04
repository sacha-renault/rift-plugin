use vizia::prelude::*;

use crate::prelude::ClapParam;

/// A function that transform struct like Map<...> into impl Lens<...>
///
/// Example:
/// ```ignore
/// // This impl Lens but we can have trouble to pass it directly into some struct
/// let lens = AppData::params.map(|p| access_fn(p).raw_value());
///
/// // This on the other hand is easier to use
/// let lens =  make_lens(AppData::params, |params| access_fn(params), |p| p.raw_value());
/// ```
pub fn make_lens<L, P, MapFn, F, R>(params: L, accessor: MapFn, f: F) -> impl Lens<Target = R>
where
    L: Lens<Target = P> + Copy,
    MapFn: 'static + Copy + Fn(&P) -> &dyn ClapParam,
    F: Fn(&dyn ClapParam) -> R + Clone + 'static,
    R: Clone + 'static,
{
    params.map(move |params| {
        let param = accessor(params);
        f(param)
    })
}
