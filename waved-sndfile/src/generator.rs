pub fn sine(sample_rate: u32, frequency: f32) -> impl std::iter::Iterator<Item = f32> {
    let mut t = 0.0_f64;
    let t_inc = 1.0_f64 / sample_rate as f64;

    let w = 2.0_f32 * std::f32::consts::PI * frequency;

    std::iter::from_fn(move || {
        let s = (w * t as f32).sin();
        t += t_inc;
        Some(s)
    })
}

pub fn square(sample_rate: u32, frequency: f32) -> impl std::iter::Iterator<Item = f32> {
    let mut t = 0.0_f64;
    let t_inc = 1.0_f64 / sample_rate as f64;

    std::iter::from_fn(move || {
        let ft = frequency * t as f32;
        let s = 2.0_f32 * (2.0_f32 * ft.floor() - (2.0_f32 * ft).floor()) - 1.0_f32;
        t += t_inc;
        Some(s)
    })
}
