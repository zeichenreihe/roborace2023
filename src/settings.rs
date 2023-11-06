use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::line_follower::LineFollower;
use crate::menu;
use crate::robot::Robot;
use crate::robot::state::RobotState;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct Settings {
	line: LineFollower,

	#[serde(skip)]
	state: RobotState,
}

impl Settings {
	fn path() -> &'static Path {
		Path::new("./robot_settings.toml")
	}

	pub(crate) fn get() -> Result<Settings> {
		let path = Self::path();

		if path.exists() {
			let string = std::fs::read_to_string(path)
				.context("Failed to read settings file")?;
			toml::from_str(&string)
				.context("Failed to parse settings")
		} else {
			println!("No settings file found, writing new settings file to {path:?}");

			let settings = Settings::default();

			let string = toml::to_string_pretty(&settings)
				.context("Failed to serialize the settings")?;
			std::fs::write(path, string)
				.context("Failed to write settings file")?;

			Ok(settings)
		}
	}

	pub(crate) fn write(self) -> Result<()> {
		let path = Self::path();

		let out = toml::to_string_pretty(&self)
			.context("Failed to serialize the settings")?;
		let read = std::fs::read_to_string(path).unwrap_or_else(|_| String::new());

		if out != read {
			println!("Updating settings file {path:?}");
			std::fs::write(path, out)
				.context("Failed to write settings file")?;
		}

		Ok(())
	}

	/// Return `Ok(true)` to end the program, `Ok(false)` otherwise.
	pub(crate) fn tick(&mut self, bot: &Robot) -> Result<bool> {
		bot.buttons.update();
		if bot.buttons.is_left() {
			std::thread::sleep(Duration::from_millis(300));
			self.next_state(bot, RobotState::InMenu)?;
		}

		match self.state {
			RobotState::Exit => {
				// try to stop the motors
				let _ = bot.left.stop();
				let _ = bot.right.stop();

				return Ok(true)
			},
			RobotState::InMenu => {
				if let Some(new_state) = menu::select(bot)? {
					self.next_state(bot, new_state)?;
				}
			},
			RobotState::Test => {
				bot.test()?;
				self.next_state(bot, RobotState::Exit)?;
			},

			RobotState::LineMeasure => {
				self.line.measure(bot)?;
				self.next_state(bot, RobotState::InMenu)?;
			},
			RobotState::LineDrive => {
				self.line.drive(bot)?;
			},
		}

		Ok(false)
	}

	pub(crate) fn next_state(&mut self, bot: &Robot, new_state: RobotState) -> Result<()> {
		match self.state {
			RobotState::LineDrive => {
				self.line.end_drive(bot)
					.context("Failed to end line drive")?;
			},
			_ => {},
		}

		match (&self.state, &new_state) {
			(_, RobotState::LineMeasure) => {},
			(_, RobotState::LineDrive) => {
				self.line.prepare_drive(bot)
					.context("Failed to prepare for line drive")?;
			},
			(_, _) => {},
		}

		self.state = new_state;
		Ok(())
	}
}