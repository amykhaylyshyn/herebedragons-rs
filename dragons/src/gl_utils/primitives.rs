use std::ffi::c_void;

use gl::types::GLenum;
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

impl<'a> Drop for Texture2d<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

#[derive(Debug)]
pub enum BufferType {
    ArrayBuffer,
    ElementArrayBuffer,
}

impl From<BufferType> for GLenum {
    fn from(buffer_type: BufferType) -> Self {
        match buffer_type {
            BufferType::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferType::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

pub struct Buffer<'a> {
    context: &'a PossiblyCurrentContext,
    handle: u32,
    buffer_type: GLenum,
}

impl<'a> Buffer<'a> {
    pub fn new(
        context: &'a PossiblyCurrentContext,
        buffer_type: BufferType,
        size: usize,
        data: *const c_void,
    ) -> Self {
        let gl_buffer_type = buffer_type.into();
        let mut handle: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut handle);
            gl::BindBuffer(gl_buffer_type, handle);
            gl::BufferData(
                gl_buffer_type,
                size.try_into().unwrap(),
                data,
                gl::STATIC_DRAW,
            );
        }

        Self {
            context,
            handle,
            buffer_type: gl_buffer_type,
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(self.buffer_type, self.handle) };
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}
