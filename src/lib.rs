//! Captcha based on SVG.
//! 
//! ## Original idea
//! 
//! [SVG绘制原理与验证码](https://blog.woooo.tech/posts/svg_1/)
//! 
//! ## Usage
//! 
//! `cargo add biosvg`
//! 
//! ```rust
//! let (answer, svg) = BiosvgBuilder::new()
//!     .length(4)
//!     .difficulty(6)
//!     .colors(vec![
//!         "#0078D6".to_string(),
//!         "#aa3333".to_string(),
//!         "#f08012".to_string(),
//!         "#33aa00".to_string(),
//!         "#aa33aa".to_string(),
//!     ])
//!     .build()
//!     .unwrap();
//! println!("answer: {}", answer);
//! println!("svg: {}", svg);
//! ```

mod model;
mod resource;
use model::Command;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use resource::{FONT_PATHS, FONT_TABLE};

/// BiosvgBuilder is a builder for generating svg captcha with random text
#[derive(Debug, Clone, Default)]
pub struct BiosvgBuilder {
    length: usize,
    difficulty: u16,
    colors: Vec<String>,
}

impl BiosvgBuilder {
    /// constructor
    pub fn new() -> BiosvgBuilder {
        BiosvgBuilder::default()
    }

    /// set length of captcha text
    pub fn length(mut self, length: usize) -> BiosvgBuilder {
        self.length = length;
        self
    }

    /// set difficulty of captcha, `difficulty` number of noise lines will be added
    pub fn difficulty(mut self, difficulty: u16) -> BiosvgBuilder {
        self.difficulty = difficulty;
        self
    }

    /// set colors of captcha text and noise lines, each color will be used randomly,
    /// please add at least 4 colors.
    /// the result of captcha will have a transparent background,
    /// so you should add colors that looks good on your website background
    pub fn colors(mut self, colors: Vec<String>) -> BiosvgBuilder {
        self.colors = colors;
        self
    }

    /// build and generate svg captcha
    pub fn build(self) -> Result<(String, String), model::PathError> {
        // generate random text with length
        let mut answer = String::new();
        let mut rng = thread_rng();
        for _ in 0..self.length {
            let index = rng.gen_range(0..FONT_TABLE.len());
            answer.push(String::from(FONT_TABLE).chars().nth(index).unwrap());
        }

        // split colors
        let mut char_colors = Vec::new();
        let mut line_colors = Vec::new();
        for color in &self.colors {
            let give_char = rng.gen_range(0..=1);
            if give_char == 1 {
                char_colors.push(color.clone());
            } else {
                line_colors.push(color.clone());
            }
        }
        let mut font_paths = Vec::new();
        for ch in answer.chars() {
            FONT_PATHS.get(ch.to_string().as_str()).map(|path| {
                let random_angle = rng.gen_range(-0.2..0.2 * std::f64::consts::PI);
                // let random_angle = random_angle + std::f64::consts::PI * 1.0;
                let random_offset = rng.gen_range(0.0..0.1 * path.width);
                let random_color = char_colors.choose(&mut rng).unwrap();
                let random_scale_x = rng.gen_range(0.8..1.2);
                let random_scale_y = rng.gen_range(0.8..1.2);
                let path = path
                    .with_color(&random_color)
                    .scale(random_scale_x, random_scale_y)
                    .rotate(random_angle)
                    .offset(0.0, random_offset);

                font_paths.push(path.clone())
            });
        }
        let mut width = 0.0;
        let mut height = 0.0;
        for path in &font_paths {
            width += path.width;
            // height = max height of all paths
            if path.height > height {
                height = path.height;
            }
        }
        width += 1.5 * height;
        let mut start_point = height * 0.55;
        let mut paths = Vec::new();
        for path in font_paths {
            let offset_x = start_point + path.width / 2.0;
            let offset_y = (height * 1.5) / 2.0;
            let mut random_splited_path = path.offset(offset_x, offset_y).random_split();
            paths.append(random_splited_path.as_mut());
            start_point += path.width + height * 0.4 / self.length as f64;
        }
        for _ in 1..self.difficulty {
            let start_x = rng.gen_range(0.0..width);
            let end_x = rng.gen_range(start_x..start_x + height);
            let start_y = rng.gen_range(0.0..height);
            let end_y = rng.gen_range(start_y..start_y + height);
            let color = line_colors.choose(&mut rng).unwrap();
            let start_command = Command {
                x: start_x,
                y: start_y,
                command_type: model::CommandType::Move,
            };
            let end_command = Command {
                x: end_x,
                y: end_y,
                command_type: model::CommandType::LineTo,
            };
            paths.push(model::Path {
                commands: vec![start_command, end_command],
                width,
                height: height / 1.5,
                color: color.clone(),
            });
        }
        paths.shuffle(&mut rng);
        let svg_content = paths
            .iter()
            .map(|path| path.to_string())
            .collect::<Vec<String>>()
            .join("");
        Ok((
            answer,
            format!(
                r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" version="1.1">{}</svg>"#,
                width,
                height * 1.5,
                width,
                height * 1.5,
                svg_content
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (answer, svg) = BiosvgBuilder::new()
            .length(4)
            .difficulty(6)
            .colors(vec![
                "#0078D6".to_string(),
                "#aa3333".to_string(),
                "#f08012".to_string(),
                "#33aa00".to_string(),
                "#aa33aa".to_string(),
            ])
            .build()
            .unwrap();
        println!("answer: {}", answer);
        println!("svg: {}", svg);
    }
}
