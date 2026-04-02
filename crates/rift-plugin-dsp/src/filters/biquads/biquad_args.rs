const Q_ORDER_2: [f32; 1] = [0.70710678];
const Q_ORDER_4: [f32; 2] = [0.54119610, 1.3065630];
const Q_ORDER_6: [f32; 3] = [0.51763809, 0.70710678, 1.9318517];
const Q_ORDER_8: [f32; 4] = [0.50979558, 0.60134489, 0.89997622, 2.5629154];
const Q_ORDER_10: [f32; 5] = [0.50623256, 0.56116312, 0.70710678, 1.1013446, 3.1962266];
const Q_ORDER_12: [f32; 6] = [
    0.50431448, 0.54119610, 0.63023621, 0.82133982, 1.3065630, 3.8306488,
];
const Q_ORDER_14: [f32; 7] = [
    0.50316379, 0.52972649, 0.59051105, 0.70710678, 0.93979296, 1.5138713, 4.4657021,
];
const Q_ORDER_16: [f32; 8] = [
    0.50241929, 0.52249861, 0.56694403, 0.64682178, 0.78815462, 1.0606777, 1.7224471, 5.1011486,
];
const Q_ORDER_18: [f32; 9] = [
    0.50190992, 0.51763809, 0.55168896, 0.61038729, 0.70710678, 0.87172340, 1.1831008, 1.9318517,
    5.7368566,
];
const Q_ORDER_20: [f32; 10] = [
    0.50154610, 0.51420760, 0.54119610, 0.58641385, 0.65754350, 0.76988452, 0.95694043, 1.3065630,
    2.1418288, 6.3727474,
];

pub const CASCADE_MAX_DEPTH: usize = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum FilterOrder {
    Two,
    Four,
    Six,
    Eight,
    Ten,
    Twelve,
    Fourteen,
    Sixteen,
    Eighteen,
    Twenty,
}

impl FilterOrder {
    /// Get the correct Q depending on the order
    /// 0 correspond to order 2, 1 to 4 etc ...
    pub fn get_q(&self, cascade_depth: usize) -> f32 {
        match self {
            FilterOrder::Two => Q_ORDER_2[cascade_depth],
            FilterOrder::Four => Q_ORDER_4[cascade_depth],
            FilterOrder::Six => Q_ORDER_6[cascade_depth],
            FilterOrder::Eight => Q_ORDER_8[cascade_depth],
            FilterOrder::Ten => Q_ORDER_10[cascade_depth],
            FilterOrder::Twelve => Q_ORDER_12[cascade_depth],
            FilterOrder::Fourteen => Q_ORDER_14[cascade_depth],
            FilterOrder::Sixteen => Q_ORDER_16[cascade_depth],
            FilterOrder::Eighteen => Q_ORDER_18[cascade_depth],
            FilterOrder::Twenty => Q_ORDER_20[cascade_depth],
        }
    }

    pub fn get_num_stages(&self) -> usize {
        match self {
            FilterOrder::Two => 1,
            FilterOrder::Four => 2,
            FilterOrder::Six => 3,
            FilterOrder::Eight => 4,
            FilterOrder::Ten => 5,
            FilterOrder::Twelve => 6,
            FilterOrder::Fourteen => 7,
            FilterOrder::Sixteen => 8,
            FilterOrder::Eighteen => 9,
            FilterOrder::Twenty => 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterMode {
    /// Lowpass with cutoff frequency
    LowPass { cutoff: f32 },
}
