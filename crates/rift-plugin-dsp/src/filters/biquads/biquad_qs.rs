// =================================================================
//              Q VALUES FOR LOW AND HIGH PASS FILTERS
// =================================================================

// Butterworth Q values for each cascade stage, by filter order.
// Ensures maximally flat magnitude response when stages are cascaded.
const PASS_Q_ORDER_2: &[f32] = &[0.70710678];
const PASS_Q_ORDER_4: &[f32] = &[0.54119610, 1.3065630];
const PASS_Q_ORDER_6: &[f32] = &[0.51763809, 0.70710678, 1.9318517];
const PASS_Q_ORDER_8: &[f32] = &[0.50979558, 0.60134489, 0.89997622, 2.5629154];
const PASS_Q_ORDER_10: &[f32] = &[0.50623256, 0.56116312, 0.70710678, 1.1013446, 3.1962266];
const PASS_Q_ORDER_12: &[f32] = &[
    0.50431448, 0.54119610, 0.63023621, 0.82133982, 1.3065630, 3.8306488,
];
const PASS_Q_ORDER_14: &[f32] = &[
    0.50316379, 0.52972649, 0.59051105, 0.70710678, 0.93979296, 1.5138713, 4.4657021,
];
const PASS_Q_ORDER_16: &[f32] = &[
    0.50241929, 0.52249861, 0.56694403, 0.64682178, 0.78815462, 1.0606777, 1.7224471, 5.1011486,
];
const PASS_Q_ORDER_18: &[f32] = &[
    0.50190992, 0.51763809, 0.55168896, 0.61038729, 0.70710678, 0.87172340, 1.1831008, 1.9318517,
    5.7368566,
];
const PASS_Q_ORDER_20: &[f32] = &[
    0.50154610, 0.51420760, 0.54119610, 0.58641385, 0.65754350, 0.76988452, 0.95694043, 1.3065630,
    2.1418288, 6.3727474,
];

pub const PASS_Q_ORDER: &[&[f32]] = &[
    PASS_Q_ORDER_2,
    PASS_Q_ORDER_4,
    PASS_Q_ORDER_6,
    PASS_Q_ORDER_8,
    PASS_Q_ORDER_10,
    PASS_Q_ORDER_12,
    PASS_Q_ORDER_14,
    PASS_Q_ORDER_16,
    PASS_Q_ORDER_18,
    PASS_Q_ORDER_20,
];
