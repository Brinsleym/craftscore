//! PNG encoding of rendered bitmaps.

/// Encodes a `width` × `height` premultiplied RGBA8 image (4 bytes per pixel, row-major) as an
/// 8-bit RGBA PNG, un-premultiplying alpha. Fails, never panics, if `rgba` has the wrong length.
pub fn encode_premultiplied(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or("image too large")?;
    if width == 0 || height == 0 || rgba.len() != expected {
        return Err(format!(
            "{} bytes is not a {width}×{height} RGBA image",
            rgba.len()
        ));
    }
    let mut straight = rgba.to_vec();
    for [r, g, b, a] in straight.as_chunks_mut::<4>().0 {
        if *a != 0 && *a != 255 {
            let alpha = u16::from(*a);
            for c in [r, g, b] {
                // Premultiplied channels never exceed alpha; min() guards bad input.
                *c = ((u16::from(*c) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
    let mut out = Vec::new();
    let mut encoder = ::png::Encoder::new(&mut out, width, height);
    encoder.set_color(::png::ColorType::Rgba);
    encoder.set_depth(::png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer
        .write_image_data(&straight)
        .map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::encode_premultiplied;

    #[test]
    fn round_trips_through_the_decoder() {
        let rgba = [255, 0, 0, 255, 64, 64, 64, 128, 0, 0, 0, 0, 10, 20, 30, 255];
        let png = encode_premultiplied(2, 2, &rgba).unwrap();
        let decoder = ::png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (2, 2));
        assert_eq!(&buf[..4], &[255, 0, 0, 255]);
        assert_eq!(&buf[4..8], &[128, 128, 128, 128]);
        assert_eq!(&buf[12..16], &[10, 20, 30, 255]);
    }

    #[test]
    fn wrong_sizes_are_errors() {
        assert!(encode_premultiplied(2, 2, &[0; 15]).is_err());
        assert!(encode_premultiplied(0, 2, &[]).is_err());
        assert!(encode_premultiplied(u32::MAX, u32::MAX, &[0; 4]).is_err());
    }
}
