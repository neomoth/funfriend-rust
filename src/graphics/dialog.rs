use super::{
	super::{
		font_manager::FontMan,
		vec2::Vec2,
		window::{Window, Windowed},
	},
	render,
};
use glfw::Context as _;

pub struct Dialog {
	text: render::Text,
	parent: Option<Box<Dialog>>,
	parent_relative_pos: Vec2,
	timer: f64,
	window: Window,
	window_size: Vec2,
}

impl Dialog {
	pub const DEFAULT_DURATION: f64 = 6.0;
	const PADDING: f64 = 10.0;

	pub fn new(
		text: &str,
		font: &str,
		position: Vec2,
		duration: f64,
		parent: Option<Box<Dialog>>,
	) -> Self {
		let sheet = FontMan::parse_bm(&std::fs::read_to_string(format!("{}.fnt", font)).unwrap());

		let (text_width, text_height, _) = FontMan::position_text(text, &sheet);

		let window_size = Vec2::new(
			text_width as f64 + Self::PADDING * 2.0,
			text_height as f64 + Self::PADDING * 2.0,
		);

		let mut window = Window::new(
			window_size.x as u32,
			window_size.y as u32,
			"!!__FUNFRIEND__!! > CHATTER",
		);

		window.handle.set_pos(
			(position.x - window_size.x / 2.0) as i32,
			(position.y - window_size.y / 2.0) as i32,
		);

		let renderer = render::Text::new(
			text.to_string(),
			font.to_string(),
			sheet,
			window_size.x as i32,
			window_size.x as i32,
		);

		let mut parent_relative_pos = Vec2::zero();
		if let Some(ref p) = parent {
			let parent_window_pos = Vec2::new_t(p.window.handle.get_pos());
			let parent_window_size = Vec2::new(parent_window_pos.x, parent_window_pos.y);
			parent_relative_pos = position - (parent_window_size / 2.0);
		}

		window.handle.make_current();
		gl::load_with(|s| window.glfw.get_proc_address_raw(s) as *const _);

		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		}

		Self {
			text: renderer,
			parent,
			parent_relative_pos,
			window_size,
			timer: duration,
			window,
		}
	}

	pub fn update_pos(&mut self) {
		if let Some(ref p) = self.parent {
			let parent_window_pos = Vec2::new_t(p.window.handle.get_pos());
			let parent_window_size = Vec2::new(parent_window_pos.x, parent_window_pos.y);
			let new_pos =
				(parent_window_pos + parent_window_size / 2.0 + self.parent_relative_pos.x
					- self.window_size / 2.0);
			self.window
				.handle
				.set_pos(new_pos.x as i32, new_pos.y as i32);
		}
	}

	pub fn render(&mut self, dt: f64) {
		self.window.handle.make_current();
		gl::load_with(|s| self.window.glfw.get_proc_address_raw(s) as *const _);
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
		self.text.render();
	}

	pub fn bump(&mut self) {
		self.parent_relative_pos.y -= self.window_size.y + 10.0;
		self.update_pos();
	}
}

impl Windowed for Dialog {
	fn update(&mut self, dt: f64) {
		tracing::info!("text timer: {}", self.timer);
		self.timer -= dt;
		if self.timer <= 0.0 {
			self.window.handle.set_should_close(true);
		}
		self.update_pos();
		self.render(dt);
	}

	fn clean_up(&mut self) {
		self.text.clean_up();
	}

	fn should_close(&self) -> bool {
		self.window.handle.should_close()
	}

	fn get_window(&mut self) -> &mut Window {
		&mut self.window
	}
}
