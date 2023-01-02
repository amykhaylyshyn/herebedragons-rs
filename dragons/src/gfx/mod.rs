use glutin::context::PossiblyCurrentContext;
use image::RgbaImage;

pub struct Texture2d<'a> {
    context: &'a PossiblyCurrentContext,
    handle: u32,
}

impl<'a> Texture2d<'a> {
    pub fn new(context: &'a PossiblyCurrentContext, image: RgbaImage) -> Self {
        let mut handle: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA.try_into().unwrap(),
                image.width().try_into().unwrap(),
                image.height().try_into().unwrap(),
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.as_ptr().cast(),
            );
        }

        Self { context, handle }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.handle);
        }
    }
}
