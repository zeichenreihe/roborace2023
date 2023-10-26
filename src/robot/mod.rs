use std::time::Duration;
use anyhow::{Context, Result};
use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::{ColorSensor, SensorPort};
use crate::robot::button::Buttons;
use crate::robot::motor::Motor;

pub(crate) mod state;
pub(crate) mod motor;
pub(crate) mod button;

#[derive(Debug)]
pub(crate) struct Robot {
	pub(crate) buttons: Buttons,

	pub(crate) color: ColorSensor,
	//pub gyro: GyroSensor,
	pub(crate) left: Motor,
	pub(crate) right: Motor,
}

impl Robot {
	pub(crate) fn new() -> Result<Robot> {
		Ok(Robot {
			buttons: Buttons::new()
				.context("Failed to get the robot buttons")?,

			color: ColorSensor::get(SensorPort::In1)
				.context("Failed to get the color sensor")?,
			//gyro: GyroSensor::get(SensorPort::In2)
			//	.context("Failed to get the gyro sensor")?,

			left: {
				let left = LargeMotor::get(MotorPort::OutA)
					.context("Failed to get the left motor")?;
				left.set_polarity(LargeMotor::POLARITY_INVERSED)?;
				left.set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;
				left.set_speed_sp(left.get_max_speed()?)?;
				Motor::new(left, "left")
			},
			right: {
				let right = LargeMotor::get(MotorPort::OutB)
					.context("Failed to get the right motor")?;
				right.set_polarity(LargeMotor::POLARITY_INVERSED)?;
				right.set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;
				right.set_speed_sp(right.get_max_speed()?)?;
				Motor::new(right, "right")
			},
		})
	}

	pub(crate) fn test(&self) -> Result<()> {
		dbg!(&self.left);
		dbg!(&self.right);

		self.left.set_speed(100f64)?;
		self.right.set_speed(100f64)?;

		self.left.start()?;
		self.right.start()?;

		std::thread::sleep(Duration::from_secs(3));

		self.left.set_speed(-100f64)?;
		self.right.set_speed(-100f64)?;

		std::thread::sleep(Duration::from_secs(4));

		self.left.set_speed(100f64)?;
		self.right.set_speed(100f64)?;

		std::thread::sleep(Duration::from_secs(1));

		self.left.stop()?;
		self.right.stop()?;

		Ok(())
	}
}