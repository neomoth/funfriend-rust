use std::cell::RefCell;
use std::ffi::CString;
use std::rc::Rc;

use gl::types::*;
use glfw::Context;

use super::super::super::{
	buddy::BuddyDefinition,
	config, glfn,
	texture::{SizedTexture, TextureBasket},
	vec2::Vec2,
	Window, FUNFRIEND_FRAG, NOP_FRAG, NOP_VERT,
};

pub struct Buddy {
	pub body_shader: GLuint,
	pub bg_shader: GLuint,
	pub vertex_array: GLuint,
	pub vertex_buffer: GLuint,
	pub body: TextureBasket,
	pub background: Option<SizedTexture>,
	pub resolution: Vec2,
}

impl Buddy {
	pub fn new(
		config: &config::Config,
		buddy: Rc<RefCell<dyn BuddyDefinition>>,
		window: &mut Window,
	) -> Self {
		let buddy = buddy.borrow();

		window.handle.make_current();
		gl::load_with(|s| window.glfw.get_proc_address_raw(s) as *const _);
		let (buddy_shader, bg_shader) = Self::init_shaders();
		let (vertex_array, vertex_buffer) = Self::init_buffers();
		let body = buddy.body();
		let background = buddy.background();

		Self {
			body_shader: buddy_shader,
			bg_shader,
			vertex_array,
			vertex_buffer,
			body,
			background,
			resolution: config.window.size,
		}
	}

	pub fn funfriend_size(&self) -> (i32, i32) {
		(self.resolution.x as i32, self.resolution.y as i32)
	}

	fn init_buffers() -> (u32, u32) {
		let vertices: [f32; 20] = [
			1.0, 1.0, 0.0, 1.0, 1.0, // top right
			1.0, -1.0, 0.0, 1.0, 0.0, // bottom right
			-1.0, -1.0, 0.0, 0.0, 0.0, // bottom left
			-1.0, 1.0, 0.0, 0.0, 1.0, // top left
		];

		let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];

		let mut vertex_array = 0;
		let mut vertex_buffer = 0;
		let mut element_buffer = 0;

		unsafe {
			gl::GenVertexArrays(1, &mut vertex_array);
			gl::GenBuffers(1, &mut vertex_buffer);
			gl::GenBuffers(1, &mut element_buffer);

			gl::BindVertexArray(vertex_array);
			gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vertices.len() * std::mem::size_of::<f32>()) as isize,
				vertices.as_ptr() as *const std::ffi::c_void,
				gl::STATIC_DRAW,
			);

			gl::VertexAttribPointer(
				0,
				3,
				gl::FLOAT,
				gl::FALSE,
				5 * std::mem::size_of::<f32>() as i32,
				std::ptr::null(),
			);
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				1,
				2,
				gl::FLOAT,
				gl::FALSE,
				5 * std::mem::size_of::<f32>() as i32,
				(3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
			);
			gl::EnableVertexAttribArray(1);

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
			gl::BufferData(
				gl::ELEMENT_ARRAY_BUFFER,
				(indices.len() * std::mem::size_of::<u32>()) as isize,
				indices.as_ptr() as *const std::ffi::c_void,
				gl::STATIC_DRAW,
			);
		}

		(vertex_array, vertex_buffer)
	}

	fn init_shaders() -> (GLuint, GLuint) {
		let ff_frag = std::str::from_utf8(FUNFRIEND_FRAG).unwrap();
		let nop_vert = std::str::from_utf8(NOP_VERT).unwrap();
		let nop_frag = std::str::from_utf8(NOP_FRAG).unwrap();
		let buddy_shader = glfn::shader(ff_frag, nop_vert);
		let bg_shader = glfn::shader(nop_frag, nop_vert);

		(buddy_shader, bg_shader)
	}

	//noinspection RsCStringPointer
	pub fn render(&mut self, dt: f64, window_width: i32, window_height: i32, window: &Window) {
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 0.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::Viewport(0, 0, window_width, window_height);
		}

		self.body.update(dt);
		let frame = self.body.texture();

		let (width, height) = (frame.width, frame.height);

		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

			if let Some(bg_texture) = &self.background {
				gl::BindTexture(gl::TEXTURE_2D, bg_texture.tex);
				gl::UseProgram(self.bg_shader);

				gl::Uniform1i(
					gl::GetUniformLocation(
						self.bg_shader,
						CString::new("texture1").unwrap().as_ptr(),
					),
					0,
				);
				gl::BindVertexArray(self.vertex_array);

				gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
			}

			gl::BindTexture(gl::TEXTURE_2D, frame.tex);
			gl::UseProgram(self.body_shader);

			gl::Uniform1i(
				gl::GetUniformLocation(
					self.body_shader,
					CString::new("texture1").unwrap().as_ptr(),
				),
				0,
			);
			gl::Uniform2f(
				gl::GetUniformLocation(
					self.body_shader,
					CString::new("funfriendSize").unwrap().as_ptr(),
				),
				self.funfriend_size().0 as f32,
				self.funfriend_size().1 as f32,
			);
			gl::Uniform2f(
				gl::GetUniformLocation(
					self.body_shader,
					CString::new("resolution").unwrap().as_ptr(),
				),
				window_width as f32,
				window_height as f32,
			);
			gl::Uniform1f(
				gl::GetUniformLocation(self.body_shader, CString::new("time").unwrap().as_ptr()),
				window.glfw.get_time() as f32,
			);

			gl::BindVertexArray(self.vertex_array);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
		}
	}

	pub fn clean_up(&self) {
		unsafe {
			gl::DeleteVertexArrays(1, &self.vertex_array);
			gl::DeleteBuffers(1, &self.vertex_buffer);
			gl::DeleteProgram(self.bg_shader);
		}
	}
}
