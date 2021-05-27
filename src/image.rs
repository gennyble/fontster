#[derive(Debug, PartialEq)]
pub enum Colors {
    RGB,
    Grey,
}

pub struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_color(width, height, (0, 0, 0))
    }

    pub fn with_color(width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        let data = [color.0, color.1, color.2].repeat(width * height);

        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_buffer(width: usize, height: usize, mut data: Vec<u8>, colors: Colors) -> Self {
        let expected_len = match colors {
            Colors::Grey => width * height,
            Colors::RGB => width * height * 3,
        };

        if data.len() != expected_len {
            panic!(
                "Expected length to be {} but it's {}",
                expected_len,
                data.len()
            );
        }

        if colors == Colors::Grey {
            // Not the fastest, but it'll do.
            let mut colordata = Vec::with_capacity(width * height * 3);
            for byte in data.into_iter() {
                colordata.extend_from_slice(&[byte, byte, byte]);
            }
            data = colordata;
        }

        Self {
            width,
            height,
            data,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn xy_to_index(&self, x: usize, y: usize) -> usize {
        (y as usize * self.width + x) * 3
    }

    pub fn draw_img(
        &mut self,
        img: Image,
        off_x: isize,
        off_y: isize,
        ignore_black: bool,
        replace_white_color: (u8, u8, u8),
    ) {
        let img_data = img.data();
        for img_y in 0..(img.height() as isize) {
            // current pixel y value
            let y = off_y + img_y;

            if y < 0 {
                // Less than 0? Could still come into bounds
                continue;
            } else if y >= self.height as isize {
                // If the pixel Y is greater than the height, it's over
                return;
            }

            for img_x in 0..(img.width() as isize) {
                // Current pixel x value
                let x = off_x + img_x;

                if x < 0 {
                    continue;
                }
                if x >= self.width as isize {
                    break;
                } else {
                    let img_index = img.xy_to_index(img_x as usize, img_y as usize);
                    let our_index = self.xy_to_index(x as usize, y as usize);

                    if ignore_black
                        && img_data[img_index] == 0
                        && img_data[img_index + 1] == 0
                        && img_data[img_index] == 0
                    {
                        continue;
                    }

                    let nrml = |c: u8| c as f32 / 255.0;
                    let lerp = |c1: u8, c2: u8, a: u8| {
                        ((nrml(c1) + nrml(a) * (nrml(c2) - nrml(c1))) * 255.0) as u8
                    };

                    self.data[our_index] = lerp(
                        self.data[our_index],
                        replace_white_color.0,
                        img_data[img_index],
                    );

                    self.data[our_index + 1] = lerp(
                        self.data[our_index + 1],
                        replace_white_color.1,
                        img_data[img_index],
                    );

                    self.data[our_index + 2] = lerp(
                        self.data[our_index + 2],
                        replace_white_color.2,
                        img_data[img_index],
                    );
                }
            }
        }
    }

    pub fn horizontal_line(&mut self, x: usize, y: usize, len: usize, color: (u8, u8, u8)) {
        for i in 0..len {
            // TODO: Check x and y are valid coordiantes
            let index = self.xy_to_index(x + i, y);

            self.data[index] = color.0;
            self.data[index + 1] = color.1;
            self.data[index + 2] = color.2;
        }
    }

    pub fn vertical_line(&mut self, x: usize, y: usize, len: usize, color: (u8, u8, u8)) {
        for i in 0..len {
            // TODO: Check x and y are valid coordiantes
            let index = self.xy_to_index(x, y + i);

            self.data[index] = color.0;
            self.data[index + 1] = color.1;
            self.data[index + 2] = color.2;
        }
    }

    pub fn rect(&mut self, x1: usize, y1: usize, width: usize, height: usize, color: (u8, u8, u8)) {
        self.vertical_line(x1, y1, height, color); //Right
        self.horizontal_line(x1, y1, width, color); //Top
        self.vertical_line(x1 + width, y1, height, color); //Left
        self.horizontal_line(x1, y1 + height, width, color); //Bottom
    }
}
