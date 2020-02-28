mod diffuse_materials;
mod hitable_sphere;
mod linear_gradient_rectangle;
mod materials;
mod simple_antialiasing;
mod simple_rectangle;
mod simple_sphere;
mod surface_normal_sphere;

pub use diffuse_materials::DiffuseMaterials;
pub use hitable_sphere::HitableSphere;
pub use linear_gradient_rectangle::LinearGradientRectangle;
pub use materials::Materials;
pub use simple_antialiasing::SimpleAntialiasing;
pub use simple_rectangle::SimpleRectangle;
pub use simple_sphere::SimpleSphere;
pub use surface_normal_sphere::SurfaceNormalSphere;

use {
    crate::{HORIZONTAL_PARTITION, VERTICAL_PARTITION},
    rayon::prelude::*,
    std::{
        fs::File,
        io::Write,
        sync::{Arc, Mutex},
    },
};

#[derive(Debug)]
pub struct Chunk {
    x: usize,
    y: usize,
    nx: usize,
    ny: usize,
    start_x: usize,
    start_y: usize,
    buffer: Vec<u8>,
}

pub trait Demo: std::marker::Sync {
    fn render(&self, buf: &mut Vec<u8>, width: usize, height: usize, samples: u8) {
        let nx = width / VERTICAL_PARTITION;
        let ny = height / HORIZONTAL_PARTITION;

        let buf = Arc::new(Mutex::new(buf));

        (0..VERTICAL_PARTITION).into_par_iter().for_each(move |j| {
            let buf = buf.clone();
            (0..HORIZONTAL_PARTITION)
                .into_par_iter()
                .for_each(move |i| {
                    let start_y = j * ny;
                    let start_x = i * nx;
                    let x = width;
                    let y = height;
                    let mut chunk = Chunk {
                        x,
                        y,
                        nx,
                        ny,
                        start_x,
                        start_y,
                        buffer: vec![0; nx * ny * 4],
                    };
                    self.render_chunk(&mut chunk, samples);

                    let mut buf = buf.lock().unwrap();

                    let mut temp_offset = 0;
                    for j in start_y..start_y + ny {
                        let real_offset = ((y - j - 1) * x + start_x) * 4;

                        buf[real_offset..real_offset + nx * 4]
                            .copy_from_slice(&chunk.buffer[temp_offset..temp_offset + nx * 4]);

                        temp_offset += nx * 4;
                    }
                })
        });
    }

    fn render_chunk(&self, chunk: &mut Chunk, samples: u8);

    fn name(&self) -> &'static str;

    fn save_as_ppm(&self, buf: &[u8], width: usize, height: usize) {
        let header = format!("P3\n{} {}\n255\n", width, height);

        let mut file = match File::create(&format!("{}-{}x{}.ppm", self.name(), width, height)) {
            Ok(file) => file,
            Err(e) => panic!("couldn't create {}: {}", self.name(), e),
        };
        file.write_all(header.as_bytes())
            .expect("error in writing file header");

        for i in buf.chunks(4) {
            match file.write_all(format!("{} {} {}\n", i[0], i[1], i[2]).as_bytes()) {
                Ok(_) => (),
                Err(e) => panic!("couldn't write to {}: {}", self.name(), e),
            }
        }
    }
}
