use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Produces a color with given RGB values. The values range from 0 to 255.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Produces a color with given RGB values. The values range from 0.0 to 1.0.
    pub fn rgb_unit(r: f64, g: f64, b: f64) -> Self {
        Self::rgb((255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8)
    }

    /// Produces a gray of the given brightness, where 0 is black and 255 is white.
    pub fn gray(brightness: u8) -> Self {
        Self::rgb(brightness, brightness, brightness)
    }

    pub fn black() -> Self {
        Self::gray(0)
    }

    pub fn white() -> Self {
        Self::gray(255)
    }

    /// Produces a color for a given hue, saturation and value.
    ///
    /// Full hue circle extends from 0.0 to 1.0, but values from outside this
    /// range all also accepted and will be mapped onto the hue circle.
    /// For example 0.1, 2.1 and -0.9 correspond to the same hue.
    ///
    /// Saturation and value are expected to be within the 0.0 to 1.0 range.
    /// If they are below 0, they will be truncated to 0, and if they are
    /// above 1, they will be truncated to 1.
    pub fn hsv(hue: f64, saturation: f64, value: f64) -> Self {
        let h = if hue < 0.0 {
            1.0 + hue.fract()
        } else {
            hue.fract()
        };
        let s = saturation.max(0.0).min(1.0);
        let v = value.max(0.0).min(1.0);

        let c = v * s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = match (h * 6.0).trunc() as i32 {
            0 => (c, x, 0f64),
            1 => (x, c, 0f64),
            2 => (0f64, c, x),
            3 => (0f64, x, c),
            4 => (x, 0f64, c),
            _ => (c, 0f64, x),
        };

        Self {
            r: ((r1 + m) * 255.0) as u8,
            g: ((g1 + m) * 255.0) as u8,
            b: ((b1 + m) * 255.0) as u8,
        }
    }

    /// Produces a white color of a given temperature.
    ///
    /// Temperature is expected to be provided in kelvin. The algorithm was
    /// devised for values between 1000K and 40,000K. Outside this range the
    /// quality of the result is not guaranteed.
    pub fn kelvin(temp: i32) -> Self {
        let temp = temp as f64 / 100.0;

        let r = if temp <= 66.0 {
            255.0
        } else {
            329.7 * (temp - 60.0).powf(-0.133)
        };

        let g = if temp <= 66.0 {
            99.47 * temp.ln() - 161.1
        } else {
            288.1 * (temp - 60.0).powf(-0.0755)
        };

        let b = if temp < 19.0 {
            0.0
        } else if temp < 66.0 {
            138.5 * (temp - 10.0).ln() - 305.0
        } else {
            255.0
        };

        Self {
            r: r.min(255.0).max(0.0) as u8,
            g: g.min(255.0).max(0.0) as u8,
            b: b.min(255.0).max(0.0) as u8,
        }
    }

    /// Produces an instance of Color from a hex color code. The code can start
    /// with a hash symbol. Both 3-digit and 6-digit codes are accepted.
    ///
    /// In case of an invalid hex code, the function will return `None`.
    pub fn from_hex_str(code: &str) -> Option<Self> {
        let code = code.trim_start_matches(|c| c == '#');
        let (r, g, b) = match code.len() {
            6 => match u32::from_str_radix(code, 16) {
                Ok(x) => ((x & 0xFF0000) >> 16, (x & 0x00FF00) >> 8, x & 0x0000FF),
                Err(_) => return None,
            },
            3 => match u32::from_str_radix(code, 16) {
                Ok(x) => (
                    ((x & 0xF00) >> 8) * 0x11,
                    ((x & 0x0F0) >> 4) * 0x11,
                    (x & 0x00F) * 0x11,
                ),
                Err(_) => return None,
            },
            _ => return None,
        };

        Some(Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        })
    }

    /// Produces a 6-digit hex code with a hash symbol at the beginning,
    /// representing the color stored in current instance of `Color`.
    pub fn to_hex_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Produces a dimmer version of the color. The dimming factor is expected
    /// to be in the range [0.0, 1.0]. Values outside of this range will be
    /// truncated.
    pub fn dim(self, factor: f64) -> Self {
        let factor = factor.max(0.0).min(1.0);
        let dim_component = |c| ((c as f64) * factor) as u8;
        Self {
            r: dim_component(self.r),
            g: dim_component(self.g),
            b: dim_component(self.b),
        }
    }

    /// Blends two colors with the default value of gamma equal to 2.
    pub fn blend(self, other: &Self) -> Self {
        self.blend_with_gamma(other, 2.0)
    }

    /// Blends two colors with the provided value of gamma.
    pub fn blend_with_gamma(self, other: &Self, gamma: f64) -> Self {
        let blend_component = |a, b| {
            let a = (a as f64) / 255.0;
            let b = (b as f64) / 255.0;
            (((a.powf(gamma) + b.powf(gamma)) / 2.0).powf(1.0 / gamma) * 255.0) as u8
        };

        Self {
            r: blend_component(self.r, other.r),
            g: blend_component(self.g, other.g),
            b: blend_component(self.b, other.b),
        }
    }

    /// Returns a color which is a linear interpolation between self and the other
    /// provided color. The second parameter determines where the result lies between
    /// self and other, 0.0 meaning the result will be equal to self,
    /// and 1.0 meaning the result will be equal to other.
    pub fn lerp(self, other: &Self, d: f64) -> Self {
        let lerp_component = |a, b| ((a as f64) * (1.0 - d) + (b as f64) * d) as u8;

        Self {
            r: lerp_component(self.r, other.r),
            g: lerp_component(self.g, other.g),
            b: lerp_component(self.b, other.b),
        }
    }
}

pub struct ColorWithAlpha {
    color: Color,
    alpha: f64,
}

impl ColorWithAlpha {
    pub fn new(color: Color, alpha: f64) -> Self {
        Self { color, alpha }
    }

    pub fn blend_with_gamma(&self, other: &Self, gamma: f64) -> Self {
        let alpha_0 = self.alpha + other.alpha * (1.0 - self.alpha);
        let blend_component = |a, b| {
            let a = (a as f64) / 255.0;
            let b = (b as f64) / 255.0;
            ((a.powf(gamma) * self.alpha + b.powf(gamma) * other.alpha * (1.0 - self.alpha))
                .powf(1.0 / gamma)
                * 255.0) as u8
        };

        Self {
            color: Color {
                r: blend_component(self.color.r, other.color.r),
                g: blend_component(self.color.g, other.color.g),
                b: blend_component(self.color.b, other.color.b),
            },
            alpha: alpha_0,
        }
    }

    pub fn blend(&self, other: &Self) -> Self {
        self.blend_with_gamma(other, 2.0)
    }

    pub fn apply_alpha(&self) -> Color {
        self.blend(&ColorWithAlpha::new(Color::black(), 1.0)).color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_to_rgb() {
        assert_eq!(Color::hsv(0.0, 0.0, 0.0), Color::rgb(0, 0, 0), "black");
        assert_eq!(
            Color::hsv(0.0, 0.0, 1.0),
            Color::rgb(255, 255, 255),
            "white"
        );
        assert_eq!(Color::hsv(0.0, 1.0, 1.0), Color::rgb(255, 0, 0), "red 0");
        assert_eq!(Color::hsv(1.0, 1.0, 1.0), Color::rgb(255, 0, 0), "red 1");
        assert_eq!(
            Color::hsv(1.0 / 3.0, 1.0, 1.0),
            Color::rgb(0, 255, 0),
            "green"
        );
        assert_eq!(
            Color::hsv(2.0 / 3.0, 1.0, 1.0),
            Color::rgb(0, 0, 255),
            "blue"
        );
        assert_eq!(
            Color::hsv(1.0 / 6.0, 1.0, 1.0),
            Color::rgb(255, 255, 0),
            "yellow"
        );
        assert_eq!(
            Color::hsv(1.0 / 2.0, 1.0, 1.0),
            Color::rgb(0, 255, 255),
            "cyan"
        );
        assert_eq!(
            Color::hsv(5.0 / 6.0, 1.0, 1.0),
            Color::rgb(255, 0, 255),
            "magenta"
        );
        assert_eq!(
            Color::hsv(-0.2, 1.0, 1.0),
            Color::hsv(0.8, 1.0, 1.0),
            "negative hue"
        );
        assert_eq!(
            Color::hsv(1.8, 1.0, 1.0),
            Color::hsv(0.8, 1.0, 1.0),
            "hue above 1"
        );
    }

    #[test]
    fn kelvin_to_rgb() {
        assert_eq!(Color::kelvin(1000), Color::rgb(255, 67, 0));
        assert_eq!(Color::kelvin(1500), Color::rgb(255, 108, 0));
        assert_eq!(Color::kelvin(2500), Color::rgb(255, 159, 70));
        assert_eq!(Color::kelvin(5000), Color::rgb(255, 228, 205));
        assert_eq!(Color::kelvin(6600), Color::rgb(255, 255, 255));
        assert_eq!(Color::kelvin(10000), Color::rgb(201, 218, 255));
    }

    #[test]
    fn blend() {
        assert_eq!(
            Color::rgb(255, 0, 0).blend(&Color::rgb(0, 255, 0)),
            Color::rgb(180, 180, 0)
        );
    }
}
