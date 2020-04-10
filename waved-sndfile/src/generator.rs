pub fn sine(sample_rate: u32, frequency: f32) -> impl std::iter::Iterator<Item = f32> {
    // FIXME: This generator is drifting a bit when using lower frequencies
    let mut t = 0.0_f32;
    let t_inc = 1.0_f32 / sample_rate as f32;
    let w = 2.0_f32 * std::f32::consts::PI * frequency;

    std::iter::from_fn(move || {
        let s = (w * t).sin();
        t = t + t_inc;
        Some(s)
    })
}
