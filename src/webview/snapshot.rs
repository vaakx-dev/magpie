use std::{future::Future, io::Cursor};

use futures::channel::oneshot;
use image::{ImageFormat as Codec, ImageResult, RgbImage, imageops::FilterType};
use webview2_com::{
    CapturePreviewCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_JPEG, ICoreWebView2,
    },
};
use windows::{
    Win32::{
        Foundation::{E_ABORT, E_OUTOFMEMORY},
        System::Com::{IStream, STATFLAG_NONAME, STATSTG, STREAM_SEEK_SET},
        UI::Shell::SHCreateMemStream,
    },
    core::{Error, Result},
};

pub const WIDTH_PX: f32 = 240.0;
pub const HEIGHT_PX: f32 = 140.0;
const STORED_WIDTH: u32 = 2 * WIDTH_PX as u32;
const STORED_HEIGHT: u32 = 2 * HEIGHT_PX as u32;
const BLANK_TOLERANCE: u8 = 6;

struct Crop {
    x: u32,
    width: u32,
    height: u32,
}

fn memory_stream() -> Result<IStream> {
    unsafe { SHCreateMemStream(None) }.ok_or_else(|| Error::from(E_OUTOFMEMORY))
}

fn contents(stream: &IStream) -> Result<Vec<u8>> {
    let mut stat = STATSTG::default();
    unsafe { stream.Stat(&mut stat, STATFLAG_NONAME) }?;
    unsafe { stream.Seek(0, STREAM_SEEK_SET, None) }?;
    let mut bytes = vec![0; stat.cbSize as usize];
    let mut read = 0;
    let length = bytes.len() as u32;
    unsafe { stream.Read(bytes.as_mut_ptr().cast(), length, Some(&mut read)) }.ok()?;
    bytes.truncate(read as usize);
    Ok(bytes)
}

pub fn capture(webview: &ICoreWebView2) -> impl Future<Output = Result<Vec<u8>>> + 'static {
    let (sender, receiver) = oneshot::channel();
    let completed = CapturePreviewCompletedHandler::create(Box::new(move |result| {
        sender.send(result).ok();
        Ok(())
    }));
    let started = memory_stream().and_then(|stream| {
        let format = COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_JPEG;
        unsafe { webview.CapturePreview(format, &stream, &completed) }?;
        Ok(stream)
    });
    async move {
        let stream = started?;
        receiver.await.map_err(|_| Error::from(E_ABORT))??;
        contents(&stream)
    }
}

fn top_of_page(width: u32, height: u32) -> Crop {
    let fitted_height = width * STORED_HEIGHT / STORED_WIDTH;
    match fitted_height <= height {
        true => Crop {
            x: 0,
            width,
            height: fitted_height,
        },
        false => {
            let fitted_width = height * STORED_WIDTH / STORED_HEIGHT;
            Crop {
                x: (width - fitted_width) / 2,
                width: fitted_width,
                height,
            }
        }
    }
}

fn painted(thumbnail: &RgbImage) -> bool {
    let first = *thumbnail.get_pixel(0, 0);
    thumbnail.pixels().any(|pixel| {
        pixel
            .0
            .iter()
            .zip(first.0)
            .any(|(channel, reference)| channel.abs_diff(reference) > BLANK_TOLERANCE)
    })
}

pub fn shrink(page: &[u8]) -> ImageResult<Option<Vec<u8>>> {
    let page = image::load_from_memory_with_format(page, Codec::Jpeg)?;
    let crop = top_of_page(page.width(), page.height());
    let thumbnail = page
        .crop_imm(crop.x, 0, crop.width, crop.height)
        .resize_exact(STORED_WIDTH, STORED_HEIGHT, FilterType::Triangle)
        .to_rgb8();
    if !painted(&thumbnail) {
        return Ok(None);
    }
    let mut jpeg = Cursor::new(Vec::new());
    thumbnail.write_to(&mut jpeg, Codec::Jpeg)?;
    Ok(Some(jpeg.into_inner()))
}
