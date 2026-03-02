/// Build `skia_safe::Color4f` from hex: `hex!(0xRRGGBB)` (opaque) or `hex!(0xRRGGBBAA)`.
#[macro_export]
macro_rules! hex {
    ($hex:expr) => {{
        let h: u32 = $hex;
        let (r, g, b, a) = if h <= 0xFF_FF_FF {
            (
                ((h >> 16) & 0xFF) as f32 / 255.0,
                ((h >> 8) & 0xFF) as f32 / 255.0,
                (h & 0xFF) as f32 / 255.0,
                1.0,
            )
        } else {
            (
                ((h >> 24) & 0xFF) as f32 / 255.0,
                ((h >> 16) & 0xFF) as f32 / 255.0,
                ((h >> 8) & 0xFF) as f32 / 255.0,
                (h & 0xFF) as f32 / 255.0,
            )
        };
        skia_safe::Color4f::new(r, g, b, a)
    }};
}
