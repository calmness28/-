use macroquad::prelude::*;

pub struct TextureCache {
    pub snake_head: Option<Texture2D>,
    pub snake_body: Option<Texture2D>,
    pub food: Option<Texture2D>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            snake_head: None,
            snake_body: None,
            food: None,
        }
    }

    pub async fn load_textures(&mut self) {
        self.snake_head = Some(Self::create_snake_head_texture().await);
        self.snake_body = Some(Self::create_snake_body_texture().await);
        self.food = Some(Self::create_food_texture().await);
    }

    async fn create_snake_head_texture() -> Texture2D {
        let size = 64;
        let mut image = Image::gen_image_color(size, size, Color::from_rgba(50, 255, 50, 255));

        for x in 40..50 {
            for y in 15..25 {
                image.set_pixel(x, y, BLACK);
            }
        }
        for x in 40..50 {
            for y in 40..50 {
                image.set_pixel(x, y, BLACK);
            }
        }

        Texture2D::from_image(&image)
    }

    async fn create_snake_body_texture() -> Texture2D {
        let size = 64;
        let image = Image::gen_image_color(size, size, Color::from_rgba(0, 200, 0, 255));
        Texture2D::from_image(&image)
    }

    async fn create_food_texture() -> Texture2D {
        let size = 64;
        let mut image = Image::gen_image_color(size, size, Color::from_rgba(255, 100, 100, 255));

        for x in 10..20 {
            for y in 10..20 {
                image.set_pixel(x, y, WHITE);
            }
        }

        Texture2D::from_image(&image)
    }
}