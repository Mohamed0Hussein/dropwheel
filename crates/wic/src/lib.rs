//! Windows Imaging Component (WIC), for the formats Windows decodes through optional
//! Microsoft Store extensions: HEIC ("HEIF Image Extensions") and AVIF ("AV1 Video
//! Extension"), and for writing HEIC ("HEVC Video Extensions").
//!
//! The whole crate is empty on other platforms.

#![cfg(windows)]

use std::sync::OnceLock;

use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_ContainerFormatHeif, GUID_WICPixelFormat32bppRGBA,
    IWICBitmapFrameEncode, IWICImagingFactory, IWICPalette, WICBitmapDitherTypeNone,
    WICBitmapEncoderNoCache, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::StructuredStorage::{CreateStreamOnHGlobal, IPropertyBag2};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    IStream, STATFLAG_NONAME, STATSTG, STREAM_SEEK_SET,
};
use windows::core::GUID;

/// Decoded pixels, 8-bit RGBA, rows packed with no padding.
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub struct Error {
    /// The HRESULT, for the log.
    pub code: i32,
    pub message: String,
    /// Windows has no codec for the format: the matching Store extension is missing.
    pub missing_codec: bool,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (0x{:08X})", self.message, self.code)
    }
}

impl std::error::Error for Error {}

/// WINCODEC_ERR_COMPONENTNOTFOUND: no installed codec handles the data.
const COMPONENT_NOT_FOUND: i32 = 0x8898_2F50_u32 as i32;

impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Self {
        let code = e.code().0;
        Error {
            code,
            message: e.message(),
            missing_codec: code == COMPONENT_NOT_FOUND,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Initialises COM for the calling thread for as long as it lives. Jobs run on their own
/// threads, so the multithreaded apartment is right; a thread that already joined another
/// apartment keeps it and WIC still works there.
struct Com {
    owned: bool,
}

impl Com {
    fn init() -> Self {
        // SAFETY: plain COM initialisation; balanced in Drop when it succeeded.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Com { owned: hr.is_ok() }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: balances the successful CoInitializeEx in `init`.
            unsafe { CoUninitialize() };
        }
    }
}

fn factory() -> Result<IWICImagingFactory> {
    // SAFETY: creating the system WIC factory has no preconditions beyond COM.
    Ok(unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)? })
}

/// Decodes the first frame of an encoded image. `max_pixels` guards against images that
/// claim absurd dimensions; nothing is allocated for those.
///
/// HEIF stores rotation as container properties that the codec applies, so the pixels
/// come back upright; EXIF orientation must not be applied on top.
pub fn decode(data: &[u8], max_side: u32, max_pixels: u64) -> Result<Bitmap> {
    let _com = Com::init();
    let factory = factory()?;
    // SAFETY: every call below is a WIC method on live interfaces. `data` outlives the
    // stream and decoder, which are dropped before this function returns, and the pixel
    // buffer is exactly `stride * height` bytes as CopyPixels requires.
    unsafe {
        let stream = factory.CreateStream()?;
        stream.InitializeFromMemory(data)?;
        let decoder = factory.CreateDecoderFromStream(
            &stream,
            std::ptr::null(),
            WICDecodeMetadataCacheOnDemand,
        )?;
        let frame = decoder.GetFrame(0)?;
        let (mut width, mut height) = (0u32, 0u32);
        frame.GetSize(&mut width, &mut height)?;
        if width == 0 || height == 0 {
            return Err(invalid("the image has no pixels"));
        }
        if width > max_side
            || height > max_side
            || u64::from(width) * u64::from(height) > max_pixels
        {
            return Err(invalid("the image is too large to open"));
        }

        let converter = factory.CreateFormatConverter()?;
        converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppRGBA,
            WICBitmapDitherTypeNone,
            None::<&IWICPalette>,
            0.0,
            WICBitmapPaletteTypeCustom,
        )?;
        let stride = width * 4;
        let len = usize::try_from(u64::from(stride) * u64::from(height))
            .map_err(|_| invalid("the image is too large to open"))?;
        let mut rgba = vec![0u8; len];
        converter.CopyPixels(std::ptr::null(), stride, &mut rgba)?;
        Ok(Bitmap {
            width,
            height,
            rgba,
        })
    }
}

/// Whether this PC can write HEIC. The HEIF container comes with "HEIF Image Extensions",
/// but the HEVC compression inside needs Microsoft's paid "HEVC Video Extensions", and a
/// missing encoder only shows when a frame is written. So try a tiny image once.
pub fn can_encode_heic() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| encode_heic(&[128; 16 * 16 * 4], 16, 16).is_ok())
}

/// Encodes 8-bit RGBA pixels as HEIC with the codec's default quality.
pub fn encode_heic(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    encode(&GUID_ContainerFormatHeif, rgba, width, height)
}

fn encode(container: &GUID, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let stride = width
        .checked_mul(4)
        .ok_or_else(|| invalid("image too wide"))?;
    let expected = u64::from(stride) * u64::from(height);
    if rgba.len() as u64 != expected {
        return Err(invalid("pixel buffer does not match the image size"));
    }
    let _com = Com::init();
    let factory = factory()?;
    // SAFETY: WIC calls on live interfaces. The bitmap copies `rgba` (CreateBitmapFromMemory
    // copies the buffer), the stream owns its memory (fDeleteOnRelease), and the read
    // buffer is sized from the stream's own length.
    unsafe {
        let stream: IStream = CreateStreamOnHGlobal(HGLOBAL::default(), true)?;
        let encoder = factory.CreateEncoder(container, std::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;

        let mut frame: Option<IWICBitmapFrameEncode> = None;
        let mut options: Option<IPropertyBag2> = None;
        encoder.CreateNewFrame(&mut frame, &mut options)?;
        let frame = frame.ok_or_else(|| invalid("the encoder gave no frame"))?;
        frame.Initialize(options.as_ref())?;
        frame.SetSize(width, height)?;

        // WriteSource converts to whatever pixel format the encoder takes.
        let bitmap = factory.CreateBitmapFromMemory(
            width,
            height,
            &GUID_WICPixelFormat32bppRGBA,
            stride,
            rgba,
        )?;
        frame.WriteSource(&bitmap, std::ptr::null())?;
        frame.Commit()?;
        encoder.Commit()?;

        let mut stat = STATSTG::default();
        stream.Stat(&mut stat, STATFLAG_NONAME)?;
        let len = usize::try_from(stat.cbSize).map_err(|_| invalid("encoded image too large"))?;
        let mut out = vec![0u8; len];
        stream.Seek(0, STREAM_SEEK_SET, None)?;
        let mut read = 0u32;
        let want = u32::try_from(len).map_err(|_| invalid("encoded image too large"))?;
        stream
            .Read(out.as_mut_ptr().cast(), want, Some(&mut read))
            .ok()?;
        out.truncate(read as usize);
        Ok(out)
    }
}

fn invalid(message: &str) -> Error {
    Error {
        code: 0,
        message: message.to_string(),
        missing_codec: false,
    }
}
