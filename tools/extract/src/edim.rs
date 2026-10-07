// ediM (Editable Media) decoder for Macromedia Director
// Based on ProjectorRays sound.cpp implementation
// ediM chunks contain Mac 'snd ' resource format with MP3 payload

use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Decode an ediM chunk to WAV file
pub fn decode_edim_to_wav(data: &[u8], output_path: &Path) -> Result<()> {
    // Parse Mac 'snd ' resource header
    let (mp3_data, sample_rate, channels, bits_per_sample) = parse_edim_header(data)?;
    
    // Decode MP3 using symphonia
    let wav_data = decode_mp3_to_wav(mp3_data, sample_rate, channels, bits_per_sample)?;
    
    // Write WAV file
    std::fs::write(output_path, wav_data)?;
    
    Ok(())
}

/// Parse the Mac 'snd ' resource header to extract MP3 payload and format info
fn parse_edim_header(data: &[u8]) -> Result<(Vec<u8>, u32, u16, u16)> {
    if data.len() < 22 {
        anyhow::bail!("ediM data too small for header");
    }
    
    let mut cursor = std::io::Cursor::new(data);
    
    // Read format (uint16)
    let format = read_u16_be(&mut cursor)?;
    
    let mut sound_command_count = 0;
    let mut sample_rate = 22050u32; // Default
    let mut channels = 1u16;
    let mut bits_per_sample = 16u16;
    
    if format == 1 {
        // Format 1: extended with data formats
        let data_format_count = read_u16_be(&mut cursor)?;
        for _ in 0..data_format_count {
            let _data_format_id = read_u16_be(&mut cursor)?;
            let _init_option = read_u32_be(&mut cursor)?;
        }
    } else {
        // Format 2: reference count
        let _reference_count = read_u16_be(&mut cursor)?;
    }
    
    // Sound commands
    sound_command_count = read_u16_be(&mut cursor)?;
    for _ in 0..sound_command_count {
        let _cmd = read_u16_be(&mut cursor)?;
        let _param1 = read_u16_be(&mut cursor)?;
        let _param2 = read_u32_be(&mut cursor)?;
    }
    
    // Sound header
    let _sample_ptr = read_u32_be(&mut cursor)?;
    let encode_dependent = read_u32_be(&mut cursor)?;
    
    sample_rate = read_u16_be(&mut cursor)? as u32;
    let _sample_rate_frac = read_u16_be(&mut cursor)?;
    let _loop_start = read_u32_be(&mut cursor)?;
    let _loop_end = read_u32_be(&mut cursor)?;
    
    let encode = read_u8(&mut cursor)?;
    let _base_frequency = read_u8(&mut cursor)?;
    
    let mut num_samples = 0u32;
    
    if encode == 0x00 {
        // Standard header
        num_samples = encode_dependent;
        channels = 1;
        bits_per_sample = 8;
    } else if encode == 0xFF || encode == 0xFD {
        // Extended header
        channels = encode_dependent as u16;
        num_samples = read_u32_be(&mut cursor)?;
        
        // AIFF sample rate (10 bytes) - skip
        let _ = read_bytes(&mut cursor, 10)?;
        
        let _marker_chunk = read_u32_be(&mut cursor)?;
        let _instrument_chunks = read_u32_be(&mut cursor)?;
        let _aes_recording = read_u32_be(&mut cursor)?;
        
        bits_per_sample = read_u16_be(&mut cursor)?;
        let _future_use1 = read_u16_be(&mut cursor)?;
        let _future_use2 = read_u32_be(&mut cursor)?;
        let _future_use3 = read_u32_be(&mut cursor)?;
        let _future_use4 = read_u32_be(&mut cursor)?;
    } else {
        anyhow::bail!("Unknown sound encode type: 0x{:02X}", encode);
    }
    
    // Skip samples
    let _skip_samples = read_u32_be(&mut cursor)?;
    
    // Remaining data is MP3 payload
    let mp3_data = cursor.get_ref()[cursor.position() as usize..].to_vec();
    
    if mp3_data.is_empty() {
        anyhow::bail!("No MP3 data found in ediM chunk");
    }
    
    Ok((mp3_data, sample_rate, channels, bits_per_sample))
}

/// Decode MP3 data to WAV format using symphonia
fn decode_mp3_to_wav(
    mp3_data: Vec<u8>,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Result<Vec<u8>> {
    use std::io::Cursor;
    
    let mss = MediaSourceStream::new(Box::new(Cursor::new(mp3_data)), Default::default());
    
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Failed to probe MP3 format")?;
    
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == symphonia::core::codecs::CODEC_TYPE_MP3)
        .context("No MP3 track found")?;
    
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .context("Failed to create MP3 decoder")?;
    
    let mut wav_buffer = Vec::new();
    let mut sample_buffer: Option<SampleBuffer<f32>> = None;
    let mut actual_rate = if sample_rate > 0 { sample_rate } else { 22050 };
    let mut actual_channels = if (1..=2).contains(&channels) { channels } else { 1 };
    
    // Decode all packets
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        
        let decoded = decoder.decode(&packet)?;
        
        if sample_buffer.is_none() {
            let spec = *decoded.spec();
            actual_rate = spec.rate;
            actual_channels = spec.channels.count().max(1) as u16;
            sample_buffer = Some(SampleBuffer::new(decoded.capacity() as u64, spec));
        }
        
        if let Some(buf) = &mut sample_buffer {
            buf.copy_interleaved_ref(decoded);
            // Convert to i16 for WAV
            let samples = buf.samples();
            for sample in samples {
                let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                wav_buffer.extend_from_slice(&s.to_le_bytes());
            }
        }
    }
    
    // Write WAV header (always 16-bit PCM since we convert samples to i16)
    let mut wav = Vec::new();
    write_wav_header(&mut wav, wav_buffer.len() as u32, actual_rate, actual_channels, 16)?;
    wav.extend_from_slice(&wav_buffer);
    
    Ok(wav)
}

/// Write WAV file header
fn write_wav_header(
    wav: &mut Vec<u8>,
    data_size: u32,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Result<()> {
    let byte_rate = sample_rate * (channels as u32) * (bits_per_sample as u32 / 8);
    let block_align = channels * (bits_per_sample / 8);
    
    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes()); // Chunk size
    wav.extend_from_slice(b"WAVE");
    
    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (PCM)
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    
    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    
    Ok(())
}

// Helper functions for reading big-endian values
fn read_u8(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16_be(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32_be(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_bytes(cursor: &mut std::io::Cursor<&[u8]>, n: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_edim() {
        // Create a minimal ediM with Format 2, no commands, standard header
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_be_bytes()); // Format 2
        data.extend_from_slice(&0u16.to_be_bytes()); // Reference count
        data.extend_from_slice(&0u16.to_be_bytes()); // Sound command count
        data.extend_from_slice(&0u32.to_be_bytes()); // Sample ptr
        data.extend_from_slice(&0u32.to_be_bytes()); // Encode dependent
        data.extend_from_slice(&22050u16.to_be_bytes()); // Sample rate
        data.extend_from_slice(&0u16.to_be_bytes()); // Sample rate frac
        data.extend_from_slice(&0u32.to_be_bytes()); // Loop start
        data.extend_from_slice(&0u32.to_be_bytes()); // Loop end
        data.extend_from_slice(&[0x00]); // Encode = standard
        data.extend_from_slice(&[0x3C]); // Base frequency
        data.extend_from_slice(&0u32.to_be_bytes()); // Skip samples
        data.extend_from_slice(b"dummy mp3 data");
        
        let result = parse_edim_header(&data);
        assert!(result.is_ok());
        let (mp3, sr, ch, bps) = result.unwrap();
        assert_eq!(sr, 22050);
        assert_eq!(ch, 1);
        assert_eq!(bps, 8);
        assert_eq!(mp3, b"dummy mp3 data");
    }
}