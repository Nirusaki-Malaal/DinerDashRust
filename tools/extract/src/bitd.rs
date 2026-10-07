// BITD (Bitmap Data), ALFA (Alpha Channel), and JPEG ediM decoders for Macromedia Director

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};

/// Decode a JPEG `ediM` chunk with an optional `ALFA` (PackBits RLE 8-bit alpha) chunk
pub fn decode_jpeg_with_alfa(
    jpeg_data: &[u8],
    alfa_data: Option<&[u8]>,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = image::load_from_memory(jpeg_data)
        .context("Failed to decode JPEG ediM chunk")?
        .to_rgba8();

    if let Some(alfa) = alfa_data {
        let _ = apply_alfa(&mut img, alfa);
    }

    Ok(img)
}

/// Apply an `ALFA` chunk (8-bit PackBits RLE alpha channel) to an RGBA image
pub fn apply_alfa(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, alfa_data: &[u8]) -> Result<()> {
    let width = img.width();
    let height = img.height();
    let pitch = ((width + 1) & !1) as u16;
    apply_alfa_with_dims(img, alfa_data, width, height, pitch)
}

/// Apply an 8-bit PackBits RLE alpha mask with explicit mask dimensions and pitch
pub fn apply_alfa_with_dims(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    alfa_data: &[u8],
    mask_width: u32,
    mask_height: u32,
    mask_pitch: u16,
) -> Result<()> {
    let effective_pitch = if mask_pitch == 0 {
        ((mask_width + 1) & !1) as u16
    } else {
        mask_pitch
    };
    let decompressed = decompress_bitd(alfa_data, mask_width, mask_height, 8, effective_pitch)?;

    let width = img.width();
    let height = img.height();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel_mut(x, y);
            if x < mask_width && y < mask_height {
                let idx = (y as usize) * (effective_pitch as usize) + (x as usize);
                p.0[3] = decompressed.get(idx).copied().unwrap_or(0);
            } else {
                p.0[3] = 0;
            }
        }
    }
    Ok(())
}

/// Decode a BITD chunk to an RGBA image
pub fn decode_bitd(
    data: &[u8],
    width: u32,
    height: u32,
    bpp: u16,
    pitch: u16,
    palette: Option<&[u8]>,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    decode_bitd_with_alfa(data, width, height, bpp, pitch, palette, None)
}

/// Decode a BITD chunk with optional ALFA chunk to an RGBA image
pub fn decode_bitd_with_alfa(
    data: &[u8],
    width: u32,
    height: u32,
    bpp: u16,
    pitch: u16,
    palette: Option<&[u8]>,
    alfa_data: Option<&[u8]>,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    decode_bitd_with_mask(
        data,
        width,
        height,
        bpp,
        pitch,
        palette,
        alfa_data.map(|a| (a, width, height, ((width + 1) & !1) as u16)),
    )
}

/// Decode a BITD chunk with optional explicit-dimension mask to an RGBA image
pub fn decode_bitd_with_mask(
    data: &[u8],
    width: u32,
    height: u32,
    bpp: u16,
    pitch: u16,
    palette: Option<&[u8]>,
    mask: Option<(&[u8], u32, u32, u16)>,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    if width == 0 || height == 0 {
        anyhow::bail!("Invalid bitmap dimensions: {}x{}", width, height);
    }

    let effective_pitch = if pitch == 0 {
        let row_bytes = (width * bpp as u32 + 7) / 8;
        ((row_bytes + 1) & !1) as u16
    } else {
        pitch
    };

    let decompressed = decompress_bitd(data, width, height, bpp, effective_pitch)?;
    let has_mask = mask.is_some();

    let mut img = match bpp {
        1 => decode_1bpp(&decompressed, width, height, effective_pitch)?,
        2 => decode_2bpp(&decompressed, width, height, effective_pitch)?,
        4 => decode_4bpp(&decompressed, width, height, effective_pitch)?,
        8 => decode_8bpp(&decompressed, width, height, effective_pitch, palette, has_mask)?,
        16 => decode_16bpp(&decompressed, width, height, effective_pitch)?,
        32 => decode_32bpp(&decompressed, width, height, effective_pitch)?,
        _ => anyhow::bail!("Unsupported bit depth: {}", bpp),
    };

    if let Some((alfa, mw, mh, mp)) = mask {
        let _ = apply_alfa_with_dims(&mut img, alfa, mw, mh, mp);
    }

    Ok(img)
}

/// PackBits RLE decompression (Macintosh variant)
fn decompress_bitd(
    data: &[u8],
    width: u32,
    height: u32,
    bpp: u16,
    pitch: u16,
) -> Result<Vec<u8>> {
    let bytes_per_pixel = (bpp + 7) / 8;
    let expected_size = (pitch as usize) * (height as usize);
    let uncompressed_size = (width as usize) * (height as usize) * (bytes_per_pixel as usize);

    if data.len() == expected_size || data.len() == uncompressed_size {
        let mut out = data.to_vec();
        if out.len() < expected_size {
            out.resize(expected_size, 0);
        }
        return Ok(out);
    }

    let mut output = Vec::with_capacity(expected_size);
    let mut i = 0;

    while i < data.len() && output.len() < expected_size {
        let header = data[i];
        i += 1;

        if header == 0x80 {
            // PackBits no-op
            continue;
        } else if header & 0x80 == 0 {
            let run_len = (header as usize) + 1;
            let end = (i + run_len).min(data.len());
            output.extend_from_slice(&data[i..end]);
            i = end;
        } else {
            let run_len = ((header ^ 0xFF) as usize) + 2;
            if i >= data.len() {
                break;
            }
            let byte = data[i];
            i += 1;
            output.extend(std::iter::repeat(byte).take(run_len));
        }
    }

    if output.len() < expected_size {
        output.resize(expected_size, 0);
    } else if output.len() > expected_size {
        output.truncate(expected_size);
    }

    Ok(output)
}

fn decode_1bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let byte_idx = (y as usize) * (pitch as usize) + (x as usize >> 3);
            let bit_idx = 7 - (x & 7);
            let bit = (data[byte_idx] >> bit_idx) & 1;
            // In Mac 1bpp, 0 is white, 1 is black
            let color = if bit == 0 { 0xFF } else { 0x00 };
            img.put_pixel(x, y, Rgba([color, color, color, 0xFF]));
        }
    }
    Ok(img)
}

fn decode_2bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let byte_idx = (y as usize) * (pitch as usize) + (x as usize >> 2);
            let shift = 2 * (3 - (x & 3));
            let color_idx = (data[byte_idx] >> shift) & 0x3;
            let color = 255 - (color_idx * 85);
            img.put_pixel(x, y, Rgba([color, color, color, 0xFF]));
        }
    }
    Ok(img)
}

fn decode_4bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let byte_idx = (y as usize) * (pitch as usize) + (x as usize >> 1);
            let shift = 4 * (1 - (x & 1));
            let color_idx = (data[byte_idx] >> shift) & 0xF;
            let color = 255 - (color_idx * 17);
            img.put_pixel(x, y, Rgba([color, color, color, 0xFF]));
        }
    }
    Ok(img)
}

fn decode_8bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
    palette: Option<&[u8]>,
    has_mask: bool,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    let default_pal = crate::cast::mac_system_palette();
    let pal = match palette {
        Some(p) if p.len() >= 768 => p,
        _ => &default_pal,
    };

    for y in 0..height {
        for x in 0..width {
            let idx = (y as usize) * (pitch as usize) + (x as usize);
            let raw_idx = data[idx] as usize;
            let r = pal[raw_idx * 3];
            let g = pal[raw_idx * 3 + 1];
            let b = pal[raw_idx * 3 + 2];
            let is_magenta = r == 255 && g == 0 && b == 255;
            let a = if !has_mask && (raw_idx == 0 || is_magenta) {
                0x00
            } else {
                0xFF
            };
            img.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    Ok(img)
}

fn decode_16bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    for y in 0..height {
        let row_start = (y as usize) * (pitch as usize);
        for x in 0..width {
            let hi = data.get(row_start + (x as usize)).copied().unwrap_or(0);
            let lo = data
                .get(row_start + (width as usize) + (x as usize))
                .copied()
                .unwrap_or(0);
            let color = ((hi as u16) << 8) | (lo as u16);

            let r = (((color >> 10) & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((color & 0x1F) as u32 * 31 / 31) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 0xFF]));
        }
    }
    Ok(img)
}

fn decode_32bpp(
    data: &[u8],
    width: u32,
    height: u32,
    pitch: u16,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut img = ImageBuffer::new(width, height);
    let stride = width as usize;

    for y in 0..height {
        let row_offset = (y as usize) * (pitch as usize);
        for x in 0..width {
            let x_idx = x as usize;
            // Director 32-bpp RLE stores each scanline as 4 consecutive byte channels: A, R, G, B
            if row_offset + stride * 4 <= data.len() {
                let a = data[row_offset + x_idx];
                let r = data[row_offset + stride + x_idx];
                let g = data[row_offset + stride * 2 + x_idx];
                let b = data[row_offset + stride * 3 + x_idx];
                let alpha = if r == 255 && g == 0 && b == 255 {
                    0x00
                } else if a == 0 && (r != 0 || g != 0 || b != 0) {
                    0xFF
                } else {
                    a
                };
                img.put_pixel(x, y, Rgba([r, g, b, alpha]));
            } else {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_literal_run() {
        let data = vec![0x02, 0xAA, 0xBB, 0xCC];
        let result = decompress_bitd(&data, 3, 1, 8, 3).unwrap();
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_decompress_repeat_run() {
        let data = vec![0xFE, 0xAA];
        let result = decompress_bitd(&data, 3, 1, 8, 3).unwrap();
        assert_eq!(result, vec![0xAA, 0xAA, 0xAA]);
    }
}