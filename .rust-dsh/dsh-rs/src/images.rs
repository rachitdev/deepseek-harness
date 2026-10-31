//! Admission and normalization of the images a person attaches to a prompt.
//!
//! An image is checked against the deployment limits, decoded in full, and
//! stored in a normalized form: a single frame, 8 bits per channel, no
//! embedded metadata, turned upright, and no larger than the normalization
//! budget. The session log holds only the resulting [`ImageRef`]; the model is
//! sent the stored bytes. This follows the TypeScript `attachment-local`
//! normalizer, with the differences listed on [`encode`].

use std::io::Cursor;

use image::metadata::Orientation;
use image::{
    ColorType, DynamicImage, ExtendedColorType, ImageDecoder, ImageFormat, ImageReader, Limits,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::attachments::AttachmentError;
use crate::types::{ImageRef, ImageSize};

/// Media types an image may declare.
pub const MEDIA_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/webp", "image/gif"];

/// JPEG qualities tried in turn until the encoding fits the byte budget.
const JPEG_QUALITIES: [u8; 3] = [85, 75, 60];

/// Longest display name kept, in UTF-16 code units, as the TypeScript store
/// counts them.
const MAX_NAME_UNITS: usize = 255;

/// What a deployment admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageLimits {
    pub max_image_bytes: u64,
    pub max_images_per_message: usize,
    pub max_message_image_bytes: u64,
    pub max_image_pixels: u64,
    pub max_image_dimension: u32,
    pub media_types: Vec<String>,
}

/// What the stored, normalized image must fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Normalization {
    /// Total pixels; a larger image is scaled down.
    pub max_pixels: u64,
    /// Longest edge, applied after the pixel budget.
    pub max_dimension: u32,
    /// Encoded size the JPEG quality ladder aims for.
    pub max_bytes: u64,
}

/// Image configuration of the `gui` plugin. The defaults are the TypeScript
/// host's.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageConfig {
    pub max_image_bytes: u64,
    pub max_images_per_message: usize,
    pub max_message_image_bytes: u64,
    pub max_image_pixels: u64,
    pub max_image_dimension: u32,
    pub media_types: Vec<String>,
    pub normalized_max_pixels: u64,
    pub normalized_max_dimension: u32,
    pub normalized_max_bytes: u64,
}

impl Default for ImageConfig {
    fn default() -> Self {
        ImageConfig {
            max_image_bytes: 20 * 1024 * 1024,
            max_images_per_message: 20,
            max_message_image_bytes: 200 * 1024 * 1024,
            max_image_pixels: 64_000_000,
            max_image_dimension: 8192,
            media_types: MEDIA_TYPES.iter().map(|t| (*t).to_string()).collect(),
            normalized_max_pixels: 2048 * 2048,
            normalized_max_dimension: 8192,
            normalized_max_bytes: 4 * 1024 * 1024,
        }
    }
}

impl ImageConfig {
    /// Refuse a value no image could satisfy, or a media type the build
    /// cannot decode.
    pub fn validate(&self) -> Result<(), String> {
        let positive = [
            ("maxImageBytes", self.max_image_bytes),
            ("maxImagesPerMessage", self.max_images_per_message as u64),
            ("maxMessageImageBytes", self.max_message_image_bytes),
            ("maxImagePixels", self.max_image_pixels),
            ("maxImageDimension", u64::from(self.max_image_dimension)),
            ("normalizedMaxPixels", self.normalized_max_pixels),
            (
                "normalizedMaxDimension",
                u64::from(self.normalized_max_dimension),
            ),
            ("normalizedMaxBytes", self.normalized_max_bytes),
        ];
        if let Some((name, _)) = positive.iter().find(|(_, value)| *value == 0) {
            return Err(format!("gui.images: {name} must be positive"));
        }
        if self.media_types.is_empty() {
            return Err("gui.images: mediaTypes must name at least one type".to_string());
        }
        if let Some(bad) = self
            .media_types
            .iter()
            .find(|candidate| !MEDIA_TYPES.contains(&candidate.as_str()))
        {
            return Err(format!(
                "gui.images: mediaTypes names {bad:?}; this build decodes {MEDIA_TYPES:?}"
            ));
        }
        Ok(())
    }

    pub fn limits(&self) -> ImageLimits {
        ImageLimits {
            max_image_bytes: self.max_image_bytes,
            max_images_per_message: self.max_images_per_message,
            max_message_image_bytes: self.max_message_image_bytes,
            max_image_pixels: self.max_image_pixels,
            max_image_dimension: self.max_image_dimension,
            media_types: self.media_types.clone(),
        }
    }

    pub fn normalization(&self) -> Normalization {
        Normalization {
            max_pixels: self.normalized_max_pixels,
            max_dimension: self.normalized_max_dimension,
            max_bytes: self.normalized_max_bytes,
        }
    }
}

impl ImageLimits {
    /// The projection the page reads to pre-check an attachment.
    pub fn projection(&self) -> serde_json::Value {
        serde_json::json!({
            "maxImageBytes": self.max_image_bytes,
            "maxImagesPerMessage": self.max_images_per_message,
            "maxMessageImageBytes": self.max_message_image_bytes,
            "maxImagePixels": self.max_image_pixels,
            "maxImageDimension": self.max_image_dimension,
            "mediaTypes": self.media_types,
        })
    }
}

/// The service key [`ImageModels`] is provided under.
pub const IMAGE_MODELS_SERVICE: &str = "image-models";

/// The model routes that accept image input. The adapters cannot say, so the
/// deployment names them; a route not listed is sent no images.
#[derive(Debug, Clone, Default)]
pub struct ImageModels(Vec<(String, String)>);

impl ImageModels {
    pub fn new(routes: impl IntoIterator<Item = (String, String)>) -> Self {
        ImageModels(routes.into_iter().collect())
    }

    pub fn accepts(&self, provider: &str, model: &str) -> bool {
        self.0.iter().any(|(p, m)| p == provider && m == model)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// An image ready to store: the normalized bytes and the reference to them.
#[derive(Debug)]
pub struct Prepared {
    pub data: Vec<u8>,
    pub image: ImageRef,
}

fn refuse(code: &'static str, message: &str) -> AttachmentError {
    AttachmentError::new(code, message)
}

/// What a header and a scan of the container say about an image.
struct Detected {
    format: ImageFormat,
    /// Width and height a viewer sees, with the EXIF orientation applied.
    width: u32,
    height: u32,
    orientation: Orientation,
    color: ColorType,
    animated: bool,
    carries_metadata: bool,
}

fn media_type_of(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Png => Some("image/png"),
        ImageFormat::Jpeg => Some("image/jpeg"),
        ImageFormat::WebP => Some("image/webp"),
        ImageFormat::Gif => Some("image/gif"),
        _ => None,
    }
}

fn is_transposed(orientation: Orientation) -> bool {
    matches!(
        orientation,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Rotate90FlipH
            | Orientation::Rotate270FlipH
    )
}

fn malformed() -> AttachmentError {
    refuse("INVALID_IMAGE", "Unsupported or malformed image data.")
}

fn decode_limits(limits: &ImageLimits) -> Limits {
    let mut decode = Limits::default();
    decode.max_image_width = Some(limits.max_image_dimension);
    decode.max_image_height = Some(limits.max_image_dimension);
    // Room for the decoded raster, the converted copy and the resized copy.
    decode.max_alloc = Some(limits.max_image_pixels.saturating_mul(4 * 3));
    decode
}

/// Read the header, apply the size limits, and decode the whole image. Both a
/// truncated file and a hostile header fail before they cost much.
fn decode(data: &[u8], limits: &ImageLimits) -> Result<(Detected, DynamicImage), AttachmentError> {
    let format = image::guess_format(data).map_err(|_| malformed())?;
    if media_type_of(format).is_none() {
        return Err(malformed());
    }
    let mut decoder = ImageReader::with_format(Cursor::new(data), format)
        .into_decoder()
        .map_err(|_| malformed())?;
    let orientation = decoder.orientation().map_err(|_| malformed())?;
    let (raw_width, raw_height) = decoder.dimensions();
    let (width, height) = if is_transposed(orientation) {
        (raw_height, raw_width)
    } else {
        (raw_width, raw_height)
    };
    if u64::from(width) * u64::from(height) > limits.max_image_pixels {
        return Err(refuse(
            "IMAGE_TOO_MANY_PIXELS",
            "Image exceeds the configured decoded-pixel limit.",
        ));
    }
    if width.max(height) > limits.max_image_dimension {
        return Err(refuse(
            "IMAGE_DIMENSION_TOO_LARGE",
            "Image exceeds the configured per-side pixel limit.",
        ));
    }
    let color = decoder.color_type();
    let scan = scan_container(format, data);
    let carries_metadata = scan.metadata
        || decoder.icc_profile().map_err(|_| malformed())?.is_some()
        || decoder.exif_metadata().map_err(|_| malformed())?.is_some()
        || decoder.xmp_metadata().map_err(|_| malformed())?.is_some();
    decoder
        .set_limits(decode_limits(limits))
        .map_err(|_| malformed())?;
    let raster = DynamicImage::from_decoder(decoder).map_err(|_| malformed())?;
    let detected = Detected {
        format,
        width,
        height,
        orientation,
        color,
        animated: scan.animated,
        carries_metadata,
    };
    Ok((detected, raster))
}

/// Facts a walk over a container's chunks reveals and a decoder does not.
#[derive(Default)]
struct Scan {
    animated: bool,
    metadata: bool,
}

/// Walk the container's chunks or segments for animation and for metadata the
/// decoder does not report (text chunks, comments, IPTC). A container this
/// cannot follow counts as carrying metadata, so it is re-encoded, not kept.
fn scan_container(format: ImageFormat, data: &[u8]) -> Scan {
    let followed = match format {
        ImageFormat::Png => scan_png(data),
        ImageFormat::Jpeg => scan_jpeg(data),
        ImageFormat::WebP => scan_webp(data),
        _ => Some(Scan::default()),
    };
    followed.unwrap_or(Scan {
        animated: false,
        metadata: true,
    })
}

fn scan_png(data: &[u8]) -> Option<Scan> {
    let mut scan = Scan::default();
    let mut at = 8;
    while at + 8 <= data.len() {
        let length = u32::from_be_bytes(data[at..at + 4].try_into().ok()?) as usize;
        let kind = &data[at + 4..at + 8];
        match kind {
            b"tEXt" | b"zTXt" | b"iTXt" | b"eXIf" | b"iCCP" => scan.metadata = true,
            b"acTL" => scan.animated = true,
            b"IEND" => return Some(scan),
            _ => {}
        }
        at = at.checked_add(12)?.checked_add(length)?;
    }
    None
}

fn scan_jpeg(data: &[u8]) -> Option<Scan> {
    let mut scan = Scan::default();
    let mut at = 2;
    while at + 4 <= data.len() {
        if data[at] != 0xff {
            return None;
        }
        let marker = data[at + 1];
        match marker {
            0xff => {
                at += 1;
                continue;
            }
            // Start of scan: the entropy-coded data follows, no more headers.
            0xda => return Some(scan),
            0xd8 | 0x01 | 0xd0..=0xd7 => {
                at += 2;
                continue;
            }
            // JFIF (E0) and Adobe (EE) describe the encoding, not the scene.
            0xe0 | 0xee => {}
            // Other application segments (EXIF, XMP, ICC, IPTC, MPF) and comments.
            0xe1..=0xef | 0xfe => scan.metadata = true,
            _ => {}
        }
        let length = u16::from_be_bytes(data[at + 2..at + 4].try_into().ok()?) as usize;
        at = at.checked_add(2)?.checked_add(length)?;
    }
    None
}

fn scan_webp(data: &[u8]) -> Option<Scan> {
    let mut scan = Scan::default();
    let mut at = 12;
    while at + 8 <= data.len() {
        let kind = &data[at..at + 4];
        let size = u32::from_le_bytes(data[at + 4..at + 8].try_into().ok()?) as usize;
        match kind {
            b"EXIF" | b"XMP " | b"ICCP" => scan.metadata = true,
            b"ANIM" | b"ANMF" => scan.animated = true,
            _ => {}
        }
        at = at
            .checked_add(8)?
            .checked_add(size.checked_add(size & 1)?)?;
    }
    Some(scan)
}

/// What an image costs a context window: about one token per 750 pixels, the
/// rate providers publish, at least one.
pub fn estimate_tokens(width: u32, height: u32) -> usize {
    ((u64::from(width) * u64::from(height)).div_ceil(750) as usize).max(1)
}

/// Dimensions within a total-pixel budget, never enlarging, rounded inward.
pub fn fit_pixels(width: u32, height: u32, max_pixels: u64) -> (u32, u32) {
    let (w, h) = (f64::from(width), f64::from(height));
    let scale = (max_pixels as f64 / (w * h)).sqrt().min(1.0);
    if scale >= 1.0 {
        return (width, height);
    }
    let budget = |long: u32, short: u32| -> (u32, u32) {
        let mut long_side = ((f64::from(long) * scale).floor() as u32).max(1);
        let short_side = |long_side: u32| {
            ((f64::from(long_side) * f64::from(short) / f64::from(long)).round() as u32).max(1)
        };
        let mut other = short_side(long_side);
        while u64::from(long_side) * u64::from(other) > max_pixels && long_side > 1 {
            long_side -= 1;
            other = short_side(long_side);
        }
        (long_side, other)
    };
    if width >= height {
        budget(width, height)
    } else {
        let (long_side, other) = budget(height, width);
        (other, long_side)
    }
}

/// The size the stored image gets: the pixel budget, then the long-edge cap.
fn target_size(width: u32, height: u32, norm: &Normalization) -> (u32, u32) {
    let (w, h) = fit_pixels(width, height, norm.max_pixels);
    let long = w.max(h);
    if long <= norm.max_dimension {
        return (w, h);
    }
    let scale = f64::from(norm.max_dimension) / f64::from(long);
    (
        ((f64::from(w) * scale).floor() as u32).max(1),
        ((f64::from(h) * scale).floor() as u32).max(1),
    )
}

fn is_eight_bit(color: ColorType) -> bool {
    matches!(
        color,
        ColorType::L8 | ColorType::La8 | ColorType::Rgb8 | ColorType::Rgba8
    )
}

/// Whether the source can be stored as it came.
fn passes_through(detected: &Detected, bytes: usize, norm: &Normalization) -> bool {
    detected.format != ImageFormat::Gif
        && !detected.animated
        && !detected.carries_metadata
        && detected.orientation == Orientation::NoTransforms
        && is_eight_bit(detected.color)
        && bytes as u64 <= norm.max_bytes
        && u64::from(detected.width) * u64::from(detected.height) <= norm.max_pixels
        && detected.width.max(detected.height) <= norm.max_dimension
}

/// Re-encode an upright raster of the target size.
///
/// A picture with transparency is stored as lossless WebP and any other as
/// JPEG at the first quality of the ladder that fits `max_bytes`, else the
/// smallest. The TypeScript store encodes transparent pictures as lossy WebP;
/// this build's encoder is lossless only, so such an image can be larger.
fn encode(
    raster: &DynamicImage,
    transparent: bool,
    norm: &Normalization,
) -> Result<(Vec<u8>, ImageFormat), AttachmentError> {
    let failed = |cause: image::ImageError| {
        AttachmentError::io(
            "ATTACHMENT_WRITE_FAILED",
            "The image could not be converted to the normalized 8-bit sRGB form.",
            cause,
        )
    };
    if transparent {
        let rgba = raster.to_rgba8();
        let mut out = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut out)
            .encode(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                ExtendedColorType::Rgba8,
            )
            .map_err(failed)?;
        return Ok((out, ImageFormat::WebP));
    }
    let rgb = raster.to_rgb8();
    let mut smallest: Option<Vec<u8>> = None;
    for quality in JPEG_QUALITIES {
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality)
            .encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .map_err(failed)?;
        if out.len() as u64 <= norm.max_bytes {
            return Ok((out, ImageFormat::Jpeg));
        }
        if smallest.as_ref().is_none_or(|best| out.len() < best.len()) {
            smallest = Some(out);
        }
    }
    smallest
        .map(|bytes| (bytes, ImageFormat::Jpeg))
        .ok_or_else(|| refuse("ATTACHMENT_WRITE_FAILED", "The image could not be encoded."))
}

/// Check one image against the limits, decode it, and normalize it.
pub fn prepare(
    data: &[u8],
    declared: &str,
    name: Option<&str>,
    limits: &ImageLimits,
    norm: &Normalization,
) -> Result<Prepared, AttachmentError> {
    if data.len() as u64 > limits.max_image_bytes {
        return Err(refuse(
            "IMAGE_TOO_LARGE",
            "Image exceeds the configured byte limit.",
        ));
    }
    if data.is_empty() {
        return Err(refuse("INVALID_IMAGE", "Image is empty."));
    }
    let (detected, mut raster) = decode(data, limits)?;
    let media_type = media_type_of(detected.format).unwrap_or_default();
    if media_type != declared {
        return Err(refuse(
            "IMAGE_TYPE_MISMATCH",
            "Declared image type does not match its bytes.",
        ));
    }
    let (stored, stored_type, width, height) = if passes_through(&detected, data.len(), norm) {
        (data.to_vec(), media_type, detected.width, detected.height)
    } else {
        raster.apply_orientation(detected.orientation);
        let (width, height) = target_size(detected.width, detected.height, norm);
        if (width, height) != (raster.width(), raster.height()) {
            raster = raster.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        }
        // A GIF always has a colour-with-alpha raster; it is transparent only
        // when some pixel is.
        let transparent = if detected.format == ImageFormat::Gif {
            raster.to_rgba8().pixels().any(|pixel| pixel.0[3] != 255)
        } else {
            raster.color().has_alpha()
        };
        let (bytes, format) = encode(&raster, transparent, norm)?;
        let stored_type = media_type_of(format).unwrap_or_default();
        verify(&bytes, stored_type, width, height)?;
        (bytes, stored_type, width, height)
    };
    let downscaled = (detected.width, detected.height) != (width, height);
    Ok(Prepared {
        image: ImageRef {
            attachment_id: format!(
                "sha256:{}",
                crate::attachments::hex(&Sha256::digest(&stored))
            ),
            media_type: stored_type.to_string(),
            bytes: stored.len() as u64,
            width,
            height,
            name: display_name(name),
            original_dimensions: downscaled.then_some(ImageSize {
                width: detected.width,
                height: detected.height,
            }),
        },
        data: stored,
    })
}

/// Prove an encoding decodes to the dimensions and type it claims.
fn verify(data: &[u8], media_type: &str, width: u32, height: u32) -> Result<(), AttachmentError> {
    let wrong = || {
        refuse(
            "ATTACHMENT_WRITE_FAILED",
            "Image normalization did not produce a single-frame 8-bit sRGB image with matching metadata.",
        )
    };
    let (read_type, read_width, read_height) = probe(data).map_err(|_| wrong())?;
    if read_type == media_type && (read_width, read_height) == (width, height) {
        return Ok(());
    }
    Err(wrong())
}

/// The type and size a header declares, without decoding the raster.
pub fn probe(data: &[u8]) -> Result<(&'static str, u32, u32), AttachmentError> {
    let format = image::guess_format(data).map_err(|_| malformed())?;
    let media_type = media_type_of(format).ok_or_else(malformed)?;
    let decoder = ImageReader::with_format(Cursor::new(data), format)
        .into_decoder()
        .map_err(|_| malformed())?;
    let (width, height) = decoder.dimensions();
    Ok((media_type, width, height))
}

/// A display name with every path removed, or none when nothing is left.
fn display_name(name: Option<&str>) -> Option<String> {
    let leaf = name?.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = leaf
        .chars()
        .filter(|c| !matches!(c, '\u{0}'..='\u{1f}' | '\u{7f}'))
        .collect();
    let mut kept = String::new();
    let mut units = 0;
    for c in cleaned
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
        .chars()
    {
        units += c.len_utf16();
        if units > MAX_NAME_UNITS {
            break;
        }
        kept.push(c);
    }
    (!kept.is_empty()).then_some(kept)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> ImageLimits {
        ImageConfig::default().limits()
    }

    fn norm() -> Normalization {
        ImageConfig::default().normalization()
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let raster = image::RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x * 7 % 256) as u8, (y * 5 % 256) as u8, 90])
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(raster)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .unwrap();
        out
    }

    fn prepared(data: &[u8], declared: &str) -> Prepared {
        prepare(data, declared, Some("a.png"), &limits(), &norm()).unwrap()
    }

    fn refusal(data: &[u8], declared: &str) -> AttachmentError {
        prepare(data, declared, None, &limits(), &norm()).unwrap_err()
    }

    #[test]
    fn a_clean_small_png_is_stored_byte_for_byte() {
        let source = png(40, 30);
        let stored = prepared(&source, "image/png");
        assert_eq!(stored.data, source);
        assert_eq!(stored.image.media_type, "image/png");
        assert_eq!((stored.image.width, stored.image.height), (40, 30));
        assert_eq!(stored.image.original_dimensions, None);
        assert_eq!(stored.image.bytes, source.len() as u64);
        assert_eq!(stored.image.name.as_deref(), Some("a.png"));
        assert_eq!(
            stored.image.attachment_id,
            format!(
                "sha256:{}",
                crate::attachments::hex(&Sha256::digest(&source))
            )
        );
    }

    #[test]
    fn an_image_over_the_pixel_budget_is_scaled_down_and_remembers_its_size() {
        let stored = prepared(&png(3000, 2000), "image/png");
        let (width, height) = (stored.image.width, stored.image.height);
        assert!(u64::from(width) * u64::from(height) <= 2048 * 2048);
        // The aspect ratio survives to within a pixel.
        let ratio = f64::from(width) / f64::from(height);
        assert!((ratio - 1.5).abs() < 0.01, "{width}x{height}");
        assert_eq!(
            stored.image.original_dimensions,
            Some(ImageSize {
                width: 3000,
                height: 2000
            })
        );
        assert_eq!(stored.image.media_type, "image/jpeg");
        let (read_type, read_width, read_height) = probe(&stored.data).unwrap();
        assert_eq!(
            (read_type, read_width, read_height),
            ("image/jpeg", width, height)
        );
    }

    #[test]
    fn a_png_with_a_text_chunk_is_rewritten_without_it() {
        let mut source = png(10, 10);
        // Insert a tEXt chunk after IHDR (8 signature + 25 IHDR bytes).
        let text: &[u8] = b"Comment\0secret location";
        let mut chunk = Vec::new();
        chunk.extend((text.len() as u32).to_be_bytes());
        chunk.extend(b"tEXt");
        chunk.extend(text);
        chunk.extend(crc32(&[b"tEXt".as_slice(), text].concat()).to_be_bytes());
        source.splice(33..33, chunk);
        assert!(scan_png(&source).unwrap().metadata);
        let stored = prepared(&source, "image/png");
        assert_ne!(stored.data, source);
        assert!(!scan_container(ImageFormat::Jpeg, &stored.data).metadata);
    }

    /// CRC-32 as PNG chunks carry it.
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    /// A JPEG with an EXIF segment declaring `orientation`.
    fn jpeg_with_orientation(width: u32, height: u32, orientation: u16) -> Vec<u8> {
        let raster = image::RgbImage::from_fn(width, height, |x, _| {
            image::Rgb([(x * 9 % 256) as u8, 40, 80])
        });
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 90)
            .encode(raster.as_raw(), width, height, ExtendedColorType::Rgb8)
            .unwrap();
        let mut tiff = Vec::new();
        tiff.extend(b"MM\0*\0\0\0\x08\0\x01");
        tiff.extend(0x0112u16.to_be_bytes());
        tiff.extend(3u16.to_be_bytes());
        tiff.extend(1u32.to_be_bytes());
        tiff.extend(orientation.to_be_bytes());
        tiff.extend([0, 0, 0, 0, 0, 0]);
        let mut segment = vec![0xff, 0xe1];
        segment.extend(((tiff.len() + 8) as u16).to_be_bytes());
        segment.extend(b"Exif\0\0");
        segment.extend(tiff);
        jpeg.splice(2..2, segment);
        jpeg
    }

    #[test]
    fn an_exif_rotated_jpeg_is_stood_upright_and_loses_its_exif() {
        let source = jpeg_with_orientation(40, 20, 6);
        let stored = prepared(&source, "image/jpeg");
        // Orientation 6 turns the picture a quarter: 40x20 is seen as 20x40.
        assert_eq!((stored.image.width, stored.image.height), (20, 40));
        assert!(!scan_jpeg(&stored.data).unwrap().metadata);
        assert_eq!(probe(&stored.data).unwrap(), ("image/jpeg", 20, 40));
    }

    #[test]
    fn limits_see_the_dimensions_a_viewer_sees() {
        let source = jpeg_with_orientation(40, 20, 6);
        let mut tight = limits();
        tight.max_image_dimension = 30;
        // 40 wide stored, but 20 wide upright: only the tall side counts.
        let error = prepare(&source, "image/jpeg", None, &tight, &norm()).unwrap_err();
        assert_eq!(error.code, "IMAGE_DIMENSION_TOO_LARGE");
    }

    #[test]
    fn a_gif_is_stored_as_a_jpeg_when_opaque() {
        let raster = image::RgbaImage::from_pixel(12, 12, image::Rgba([10, 200, 30, 255]));
        let mut gif = Vec::new();
        image::DynamicImage::ImageRgba8(raster)
            .write_to(&mut Cursor::new(&mut gif), ImageFormat::Gif)
            .unwrap();
        let stored = prepared(&gif, "image/gif");
        assert_eq!(stored.image.media_type, "image/jpeg");
    }

    #[test]
    fn a_transparent_png_keeps_its_transparency_as_webp() {
        let raster = image::RgbaImage::from_fn(16, 16, |x, _| {
            image::Rgba([200, 0, 0, if x < 8 { 0 } else { 255 }])
        });
        let mut source = Vec::new();
        image::DynamicImage::ImageRgba8(raster)
            .write_to(&mut Cursor::new(&mut source), ImageFormat::Png)
            .unwrap();
        // Clean and small, so it passes through unchanged.
        assert_eq!(prepared(&source, "image/png").image.media_type, "image/png");
        let mut tiny = norm();
        tiny.max_pixels = 64;
        let stored = prepare(&source, "image/png", None, &limits(), &tiny).unwrap();
        assert_eq!(stored.image.media_type, "image/webp");
        let back = image::load_from_memory(&stored.data).unwrap().to_rgba8();
        assert!(back.pixels().any(|p| p.0[3] == 0), "transparency survives");
    }

    #[test]
    fn refusals_name_the_reason_the_page_routes_on() {
        let small = png(8, 8);
        assert_eq!(refusal(b"", "image/png").code, "INVALID_IMAGE");
        assert_eq!(refusal(b"hello", "image/png").code, "INVALID_IMAGE");
        assert_eq!(refusal(&small, "image/jpeg").code, "IMAGE_TYPE_MISMATCH");
        assert_eq!(
            refusal(&small[..small.len() - 20], "image/png").code,
            "INVALID_IMAGE"
        );
        let mut tight = limits();
        tight.max_image_bytes = 10;
        assert_eq!(
            prepare(&small, "image/png", None, &tight, &norm())
                .unwrap_err()
                .code,
            "IMAGE_TOO_LARGE"
        );
        let mut tight = limits();
        tight.max_image_pixels = 50;
        assert_eq!(
            prepare(&small, "image/png", None, &tight, &norm())
                .unwrap_err()
                .code,
            "IMAGE_TOO_MANY_PIXELS"
        );
    }

    #[test]
    fn a_bmp_is_not_an_image_this_build_decodes() {
        // The BMP magic number and a plausible header; no codec is built for it.
        let mut bmp = b"BM".to_vec();
        bmp.extend([0u8; 64]);
        assert_eq!(refusal(&bmp, "image/png").code, "INVALID_IMAGE");
    }

    #[test]
    fn fit_pixels_matches_the_typescript_rounding() {
        // 3000x2000 into 2048*2048: scale = sqrt(4194304/6000000).
        assert_eq!(fit_pixels(3000, 2000, 2048 * 2048), (2508, 1672));
        assert_eq!(fit_pixels(2000, 3000, 2048 * 2048), (1672, 2508));
        assert_eq!(fit_pixels(10, 10, 2048 * 2048), (10, 10));
        assert_eq!(fit_pixels(1000, 1, 100), (100, 1));
    }

    #[test]
    fn a_long_edge_cap_applies_after_the_pixel_budget() {
        let mut capped = norm();
        capped.max_pixels = u64::MAX;
        capped.max_dimension = 100;
        assert_eq!(target_size(400, 200, &capped), (100, 50));
    }

    #[test]
    fn a_display_name_loses_its_path_and_control_characters() {
        assert_eq!(display_name(None), None);
        assert_eq!(display_name(Some("../a/b.png")).as_deref(), Some("b.png"));
        assert_eq!(
            display_name(Some("C:\\x\\y z.png")).as_deref(),
            Some("y z.png")
        );
        assert_eq!(display_name(Some("  \u{7}  ")), None);
        assert_eq!(
            display_name(Some(&"x".repeat(300))).map(|n| n.len()),
            Some(255)
        );
    }

    #[test]
    fn configuration_refuses_what_no_image_could_satisfy() {
        let mut config = ImageConfig::default();
        assert!(config.validate().is_ok());
        config.max_image_bytes = 0;
        assert!(config.validate().unwrap_err().contains("maxImageBytes"));
        let mut config = ImageConfig {
            media_types: vec!["image/bmp".to_string()],
            ..ImageConfig::default()
        };
        assert!(config.validate().unwrap_err().contains("image/bmp"));
        config.media_types.clear();
        assert!(config.validate().unwrap_err().contains("mediaTypes"));
    }
}
