use bitcode::{Decode, Encode};
use std::ops::{Add, Div, Sub};

#[derive(Decode, Encode, Debug, Clone, PartialEq, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from_string(vec3_string: String) -> Vector3 {
        let pos_vec: Vec<&str> = vec3_string.split(",").collect();
        Vector3::new(
            pos_vec[0].parse().unwrap(),
            pos_vec[1].parse().unwrap(),
            pos_vec[2].parse().unwrap(),
        )
    }
}

impl Vector3 {
    pub fn add_to(&mut self, delta: Vector3) -> Self {
        self.x += delta.x;
        self.y += delta.y;
        self.z += delta.z;

        self.to_owned()
    }

    pub fn scale(&mut self, factor: f32) -> Self {
        self.x = self.x * factor;
        self.y = self.y * factor;
        self.z = self.z * factor;

        self.to_owned()
    }

    pub fn to_string(self) -> String {
        format!("{:.2},{:.2},{:.2}", self.x, self.y, self.z)
    }
}

#[derive(Decode, Encode, Debug, Clone, PartialEq, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn zero() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub fn add(vec: Vector2, delta: Vector2) -> Vector2 {
        Vector2 {
            x: vec.x + delta.x,
            y: vec.y + delta.y,
        }
    }

    pub fn from_string(vec2_string: String) -> Vector2 {
        let pos_vec: Vec<&str> = vec2_string.split(",").collect();
        Vector2::new(pos_vec[0].parse().unwrap(), pos_vec[1].parse().unwrap())
    }
}

impl Vector2 {
    pub fn add_to(&mut self, delta: Vector2) -> Self {
        self.x += delta.x;
        self.y += delta.y;

        self.clone()
    }

    pub fn scale(&mut self, factor: f32) -> Self {
        self.x = self.x * factor;
        self.y = self.y * factor;

        self.clone()
    }

    pub fn to_string(self) -> String {
        format!("{:.2},{:.2}", self.x, self.y)
    }

    pub fn rotate(&mut self, radians: f32) -> Vector2 {
        let cos_theta = radians.cos();
        let sin_theta = radians.sin();

        Vector2 {
            x: self.x * cos_theta - self.y * sin_theta,
            y: self.x * sin_theta + self.y * cos_theta,
        }
    }

    pub fn multiply(&mut self, factor: f32) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn to_vector3(&mut self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: 0.0,
            z: self.y,
        }
    }

    pub fn normalize(&mut self) -> Vector2 {
        let length = (self.x * self.x + self.y * self.y).sqrt();
        if length == 0.0 {
            Vector2::zero()
        } else {
            self.x /= length;
            self.y /= length;
            *self
        }
    }

    pub fn distance(&self, other: &Vector2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f32> for Vector2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Vector2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
