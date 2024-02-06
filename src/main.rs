extern crate sdl2;

use rand::Rng;
use cond_utils::Between;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::thread;
use std::vec;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use core::panic;
use std::time::Duration;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use std::sync::mpsc::{self, Sender};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

//TODO: cannot draw to a canvas when multitherading in sdl2, possibly get a new canvas library and use that to draw pixels to the screen.
// ! new libraries : piston2d-graphics, rust bindings for SFML
// ! idea: do a pixel filter that draws 25% of pixels when moving, this will speed up alot for "real time" rendering

#[derive(PartialEq, Clone)]
struct Sphere {
    pub radius: f64,
    pub center: Vec<f64>,
    pub color: Vec<u8>,
    pub specularity: f64,
    pub reflectivity: f64
}
impl Sphere {
    pub fn new(radius: f64, center: Vec<f64>, color: Vec<u8>, specularity: f64, reflectivity: f64) -> Sphere {
        Sphere {
            radius: radius,
            center: center,
            color: color,
            specularity: specularity,
            reflectivity: reflectivity
        }
    }
}

struct Light {
    pub typ: String,
    pub intensity: f64,
    pub position: Vec<f64>,
    pub direction: Vec<f64>
}
impl Light {
    pub fn new (intensity: f64, typ: String, position: Vec<f64>, direction: Vec<f64> ) -> Light {
        Light {
            intensity: intensity,
            typ: typ,
            position: position,
            direction: direction,
        }
    }
}


fn put_pixel(canvas: &mut Canvas<Window>,x: i32, y:i32, color: &Vec<u8>) {
    let window_width_half: i32 = (WIDTH/2) as i32;
    let window_height_half: i32 = (HEIGHT/2) as i32;
    //error handling for draw position out of range
    if x > window_width_half || x < -window_width_half {
        println!("invalid x = {}", x);
        panic!("x co-ordinate outside range")
    }
    if y > window_height_half || y < -window_height_half {
        println!("invalid y = {}", y);
        panic!("y co-ordinate outside range")
    }
    //convert cartesian to computer
    let canvas_x: i32 = (x + window_width_half) as i32;
    let canvas_y: i32 = (window_height_half - y) as i32;
    //draw pixel
    canvas.set_draw_color(Color::RGB(color[0], color[1], color[2]));
    let point = Point::new(canvas_x, canvas_y);

    // ! cannot use draw point anymore

    canvas.draw_point(point).expect("could not draw point");
}
// TODO: replace all tuples with lists as tuples are bad practice for same typing
// TODO: finish dot product function and then finish sphere ray intersection function.

pub fn main() {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("quantamax", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let res_multiplier: u32 = 1;
    let resolution: Vec<u32> = vec![WIDTH * res_multiplier, HEIGHT * res_multiplier]; 
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(resolution[0], resolution[1]).expect("could not set res");
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut render_texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24,WIDTH + 1, HEIGHT + 1).map_err(|e| e.to_string()).unwrap();
    //let mut surface = window.surface(&event_pump);

    let window_width_half: i32 = (WIDTH/2) as i32;
    let window_height_half: i32 = (HEIGHT/2) as i32;
    let sphere1 = Sphere::new(1.0, vec![0.0,-1.5, 1.5], vec![225,0,0] , -1.0, 0.5);
    let sphere2 = Sphere::new(1.0, vec![-1.5,-0.5,1.7], vec![255,192,203], 500.0, 0.5);
    let sphere3 = Sphere::new(1.0, vec![1.5,-0.5, 1.7], vec![0,0,225], 500.0, 0.5);
    let shere5 = Sphere::new(5000.0, vec![0.0,-5001.0,0.0], vec![225,225,0], -1.0, 0.2);
    let spheres = vec![sphere2,sphere1,sphere3,shere5];

    let light = Light::new(0.7, String::from("Point"), vec![-5.0,1.0,0.0], vec![0.0,0.0,0.0]);
    let light2 = Light::new(0.7, String::from("Point"), vec![5.0,1.0,0.0], vec![-1.0, -9.0, -2.0]); 
    let light3 = Light::new(0.2, String::from("Ambient"), vec![0.0,0.0,0.0], vec![0.0,0.0,0.0]);
    let lights: Vec<Light> = vec![light, light2, light3];

    let fov: f64 = 180 as f64;
    let view_width = (resolution[0] as f64) * fov;
    let view_height = (resolution[1] as f64) * fov;
    let mut camera_position = vec![0.0,0.0,-1.0];

    let camera_rotation_y: f64 = 90.0;
    let camera_rotation_x: f64 = 90.0;
    let camera_rotation_z: f64 = 0.0;

    let z_rotation_matrix: Vec<Vec<f64>> =vec![vec![f64::cos(camera_rotation_z).round(), -f64::sin(camera_rotation_z).round(), 0.0],
                                               vec![f64::sin(camera_rotation_z).round(), f64::cos(camera_rotation_z).round(), 0.0],
                                               vec![0.0,0.0,1.0]]; 


    let mut r_pressed: bool = true;
    let mut render_chance = 1000;
    let mut r_change_safety = true;
    'running: loop {
        println!("{}", r_pressed);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {keycode: Some(Keycode::R), ..} => {
                    if r_change_safety {
                        if r_pressed {
                            render_chance = 20;
                            r_pressed = false;
                        }
                        if r_pressed == false {
                            render_chance = 1000;
                            r_pressed = true;
                        }
                    }
                    r_change_safety = false;
                },
                Event::KeyUp {keycode: Some(Keycode::R), ..} => {
                    r_change_safety = true;
                },
                Event::KeyDown {keycode: Some(Keycode::W), ..} => {
                    camera_position[2] += 0.1
                },
                Event::KeyDown {keycode: Some(Keycode::S), ..} => {
                    camera_position[2] -= 0.1
                },



                _ => {}
            }
        }

        //canvas.set_draw_color(Color::RGB(225, 225, 225));
        canvas.clear();
        /* 
        let (render_width, render_height) = canvas.output_size().unwrap();
        let mut x_as_float: f32;
        let mut y_as_float: f32;
        let mut width_as_float: f32;
        let mut height_as_float: f32;
        let mut r: f32;
        let mut g: f32;
        let mut ir: u8;
        let mut ig: u8;
        
        
        for y in 0..render_height {
            for x in 0..render_width {
                x_as_float = x as f32;
                y_as_float = y as f32;
                width_as_float = (render_width - 1) as f32;
                height_as_float = (render_height - 1) as f32;
                r = x_as_float / width_as_float;
                g = y_as_float / height_as_float;

                ir = (225.0 * r).round() as u8;
                ig = (225.0 * g).round() as u8;
                //println!("{} {}",ir, ig);
                canvas.set_draw_color(Color::RGB(ir, ig, 0));
                let point = Point::new(x.try_into().unwrap(), y.try_into().unwrap());
                canvas.draw_point(point).expect("could not draw point");
            }
        }
        */

        // ! a pixel buffer can be an array of u8's, in order R, G, B, ALPHA. first 4 are (0,0)
        

        let multithreading = true;
        if multithreading == false {
            render_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for x in -window_width_half..window_width_half {
                    for y in -window_height_half..window_height_half {
                        let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                        let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                        let canvas_coords = transfer_coords(x, y);      
                        let offset = canvas_coords.1 as usize * pitch as usize + canvas_coords.0 as usize * 3;
                        //println!("{:?}",  canvas_coords);
                        //println!("{}", pitch);
                        //println!("{:?}",buffer);
                        buffer[offset] = color[0];
                        buffer[offset + 1] = color[1];
                        buffer[offset + 2] = color[2];
                        //buffer[offset + 3] = 225;
                        //println!("made a round")
                   }
                }
            }).expect("what?"); 
            canvas.copy(&render_texture, None, Rect::new(0, 0, WIDTH, HEIGHT));

            /* 
            for x in -window_width_half..window_width_half {
                for y in -window_height_half..window_height_half {
                    let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                    let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                    put_pixel(&mut canvas, x, y, &color);                    
               }
            }
            */

            //canvas.draw_points(9);
        }
        if multithreading == true {
            //create channels for communication between threads
            let (tx, rx) = mpsc::channel();

            //spawn multiple threads to update different sections of the texture
            let handles: Vec<_> = (0..20)
            .map(|i| {
                let tx = tx.clone();
                let render_chance = render_chance.clone();
                let mut camera_position = camera_position.clone();
                thread::spawn(move || {
                    update_texture_region(i, tx, render_chance, &mut camera_position);
                })
            }).collect();

            for handle in handles {
                handle.join().unwrap();
            }
            //recieve updates from threads and apply to texture
            for update in rx.try_iter() {
                render_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for (x, y, color) in update {
                        let offset = y as usize * pitch as usize + x as usize * 3;
                        //println!("{:?}",  canvas_coords);
                        //println!("{}", pitch);
                        //println!("{:?}",buffer);
                        buffer[offset] = color.0;
                        buffer[offset + 1] = color.1;
                        buffer[offset + 2] = color.2;
                        //buffer[offset + 3] = 225;
                    }
                }).expect("why error?");
            }
            canvas.copy(&render_texture, None, Rect::new(0, 0, WIDTH, HEIGHT)).expect("could not draw");   
            canvas.present();

            for _ in 0..20{
                tx.send(Vec::new()).unwrap();
            }


            /*
            let mut handles = vec![];
            let num_of_threads =1;
            let texture_mutex = Arc::new(Mutex::new(render_texture));
            for i in 0..num_of_threads {
                let mut text = Arc::clone(&texture_mutex);
                let handle = thread::spawn(move || {
                    let mut texture = text.lock().unwrap();
                    let mut new = *texture;
                    new.with_lock(None, |buffer: &mut[u8], pitch: usize| {

                    });
                });
                handles.push(handle)
            }
            for handle in handles {
                handle.join().unwrap();
            }
            render_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let mut handles = vec![];
                let num_of_threads =1;
                for i in 0..num_of_threads {
                    let handle = thread::spawn(|| {
                        
                    });
                    handles.push(handle)
                }
                for handle in handles {
                    handle.join().unwrap();
                }


                for x in -window_width_half..window_width_half {
                    for y in -window_height_half..window_height_half {
                        let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                        let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                        let canvas_coords = transfer_coords(x, y);      
                        let offset = canvas_coords.1 as usize * pitch as usize + canvas_coords.0 as usize * 3;
                        //println!("{:?}",  canvas_coords);
                        //println!("{}", pitch);
                        //println!("{:?}",buffer);
                        buffer[offset] = color[0];
                        buffer[offset + 1] = color[1];
                        buffer[offset + 2] = color[2];
                        buffer[offset + 3] = 225;
                        //println!("made a round")
                   }
                }
            }).expect("what?"); 
            let draw_rect = Rect::new(0, 0, WIDTH, HEIGHT);
            canvas.copy(&render_texture, None, Some(draw_rect)).expect("could not copy texture");
            /* 
            let mutex_width_half = Mutex::new(window_width_half);
            let mutex_height_half = Mutex::new(window_height_half);
            let handle = thread::spawn(move || {
                for x in -window_width_half..window_width_half {
                    for y in -window_height_half..window_height_half {
                        let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                        let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                        //put_pixel(&mut *canvas_ref, x, y, color);
                   }
                }
            });
            handle.join().unwrap();
            */
            */
        }
        canvas.copy(&render_texture, None, Rect::new(0, 0, WIDTH, HEIGHT));        
        println!("done");
        //put_pixel(&mut canvas, 1, 1, vec![225, 0,0]);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        //break;
    }
    ::std::thread::sleep(Duration::new(10, 1_000_000_000u32 / 60));
}

fn update_texture_region(thread_id: usize, tx: Sender<Vec<(u32, u32, (u8, u8, u8))>>, render_chance_max: i32, camera_position: &mut Vec<f64>){
    // --temp--
    //define variables
    let res_multiplier: u32 = 1;
    let resolution: Vec<u32> = vec![WIDTH * res_multiplier, HEIGHT * res_multiplier]; 

    let window_width_half: i32 = (WIDTH/2) as i32;
    let window_height_half: i32 = (HEIGHT/2) as i32;
    let sphere1 = Sphere::new(1.0, vec![0.0,-1.5, 1.5], vec![225,0,0] , -1.0, 0.5);
    let sphere2 = Sphere::new(1.0, vec![-1.5,-0.5,1.7], vec![255,192,203], 500.0, 0.5);
    let sphere3 = Sphere::new(1.0, vec![1.5,-0.5, 1.7], vec![0,0,225], 500.0, 0.5);
    let shere5 = Sphere::new(5000.0, vec![0.0,-5001.0,0.0], vec![225,225,0], -1.0, 0.2);
    let spheres = vec![sphere2,sphere1,sphere3,shere5];

    let light = Light::new(0.7, String::from("Point"), vec![-5.0,1.0,0.0], vec![0.0,0.0,0.0]);
    let light2 = Light::new(0.7, String::from("Point"), vec![5.0,1.0,0.0], vec![-1.0, -9.0, -2.0]); 
    let light3 = Light::new(0.2, String::from("Ambient"), vec![0.0,0.0,0.0], vec![0.0,0.0,0.0]);
    let lights: Vec<Light> = vec![light, light2, light3];

    let fov: f64 = 180 as f64;
    let view_width = (resolution[0] as f64) * fov;
    let view_height = (resolution[1] as f64) * fov;

    let camera_rotation_y: f64 = 90.0;
    let camera_rotation_x: f64 = 90.0;
    let camera_rotation_z: f64 = 0.0;

    let num_of_threads = 20;
    let render_width = WIDTH/num_of_threads;
    let half_render_width = render_width/2;

    let z_rotation_matrix: Vec<Vec<f64>> =vec![vec![f64::cos(camera_rotation_z).round(), -f64::sin(camera_rotation_z).round(), 0.0],
                                               vec![f64::sin(camera_rotation_z).round(), f64::cos(camera_rotation_z).round(), 0.0],
                                               vec![0.0,0.0,1.0]]; 


    let mut update: Vec<(u32, u32, (u8, u8, u8))> = Vec::new();
    //width = 500
    //window_width_half = 250
    let starting_point = (-window_width_half + (render_width * (thread_id) as u32) as i32);
    let max_negate_id = (num_of_threads - (thread_id + 1) as u32) as i32;
    let end_point = (window_width_half - (render_width * (max_negate_id) as u32) as i32);

    let mut rng = rand::thread_rng();
    for x in starting_point as i32..end_point {
        for y in -window_height_half..window_height_half {
            let color: Vec<u8>;
            let render_chance = rng.gen_range(0..1000);
            if render_chance < render_chance_max {
                let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                //println!("{:?}", color);
                //println!("{:?}", color);
            }
            else  {
                color = vec![0,0,0];
            }
            let canvas_coords = transfer_coords(x, y);

            update.push((canvas_coords.0 as u32, canvas_coords.1 as u32, (color[0], color[1], color[2])));
       }
    }


    /*
    if thread_id == 0 {
        for x in starting_point as i32..end_point {
            for y in -window_height_half..window_height_half {
                let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                //println!("{:?}", color);
                //println!("{:?}", color);
                let canvas_coords = transfer_coords(x, y);

                update.push((canvas_coords.0 as u32, canvas_coords.1 as u32, (color[0], color[1], color[2])));
           }
        }
    }
    else if thread_id == 1 {
        for x in -100..0 {
            for y in -window_height_half..window_height_half {
                let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                //println!("{:?}", color);
                //println!("{:?}", color);
                let canvas_coords = transfer_coords(x, y);

                update.push((canvas_coords.0 as u32, canvas_coords.1 as u32, (color[0], color[1], color[2])));
           }
        }
    }

    if thread_id == 2 {
        for x in 0..100 {
            for y in -window_height_half..window_height_half {
                let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                //println!("{:?}", color);
                //println!("{:?}", color);
                let canvas_coords = transfer_coords(x, y);

                update.push((canvas_coords.0 as u32, canvas_coords.1 as u32, (color[0], color[1], color[2])));
           }
        }
    }
    else if thread_id == 3 {
        for x in 100..200 {
            for y in -window_height_half..window_height_half {
                let view = canvas_to_viewport(x as f64, y as f64, view_width, view_height as f64, 1.0, &z_rotation_matrix);
                let color = trace_ray(&camera_position, view, 0.0, f64::INFINITY, &lights, &spheres, 3.0);
                //println!("{:?}", color);
                //println!("{:?}", color);
                let canvas_coords = transfer_coords(x, y);

                update.push((canvas_coords.0 as u32, canvas_coords.1 as u32, (color[0], color[1], color[2])));
           }
        }
    }
    */
    //println!("{:?}", update);

    tx.send(update).unwrap();
    //thread::sleep(Duration::from_millis(50));

        
}

fn transfer_coords(x: i32, y: i32) -> (i32, i32) {
    let window_width_half: i32 = (WIDTH/2) as i32;
    let window_height_half: i32 = (HEIGHT/2) as i32;
    let canvas_x: i32 = (x + window_width_half) as i32;
    let canvas_y: i32 = (window_height_half - y) as i32;
    return (canvas_x, canvas_y)
}

fn trace_ray(camera_origin: &Vec<f64>, view: Vec<f64>, tmin: f64, tmax: f64, lights: &Vec<Light>, spheres: &Vec<Sphere>, recursion_depth: f64) -> Vec<u8> {
    let mut closest_sphere: Option<&Sphere> = None;
    let mut closest_t: f64 = f64::INFINITY;
    let mut intersects: Vec<f64>;
    for sphere in spheres{
        intersects = intersect_ray_sphere(&camera_origin, &view, sphere); //two intersections
        if intersects[0].within(tmin, tmax) && intersects[0] < closest_t {
            closest_t = intersects[0];
            closest_sphere = Some(sphere);
        }
        if intersects[1].within(tmin, tmax) && intersects[1] < closest_t {
            closest_t = intersects[1];
            closest_sphere = Some(sphere)
        }
    }
    if closest_sphere == None {
        return vec![0, 0, 0]
    }
    //assert_eq!(intersects,vec![0.0, 0.0]);
    let unwraped_sphere = closest_sphere.unwrap();
    let intersection: Vec<f64> = vec3_addition(&camera_origin, &vec3_multiply_by_float(&view, closest_t));
    let mut normal = vec3_negation(&intersection, &unwraped_sphere.center);
    normal = vec3_divide_by_float(&normal, vec3_length(&normal));
    let light_intensity = compute_lighting(&intersection, &normal, lights, vec3_multiply_by_float(&view, -1.0), unwraped_sphere.specularity, spheres);
    
    let local_color = multiply_color_by_float(&unwraped_sphere.color, light_intensity);

    //compute reflection
    let reflectivity = unwraped_sphere.reflectivity;

    if recursion_depth < 0.0 || reflectivity == 0.0 {
        return local_color
    }

    //pythagoras
    let opposite_ray = vec3_multiply_by_float(&view, -1.0);
    let reflection_with_view = vec3_multiply_by_float(&normal, 2.0 * dot_product(&normal, &opposite_ray));
    let reflect_ray = vec3_negation(&reflection_with_view, &opposite_ray);


    let reflected_color = trace_ray(&intersection, reflect_ray, 0.001, f64::INFINITY, lights, spheres, recursion_depth - 1.0);
    //factor in reflectivity
    let real_local = multiply_color_by_float(&local_color, 1.0-reflectivity);
    let real_reflective = multiply_color_by_float(&reflected_color, reflectivity);
    let mut global_color = vec![0,0,0];
    for i in 0..real_local.len() {
        global_color[i] = real_local[i] + real_reflective[i]
    }
    return global_color;
}
/* 
fn find_intersects(camera_origin: &Vec<f64>, view: &Vec<f64>, tmin: f64, tmax: f64, spheres: & mut Vec<Sphere>) -> (Option<&mut Sphere>, f64) {
    let mut closest_sphere: Option<& mut Sphere> = None;
    let mut closest_t: f64 = f64::INFINITY;
    let mut intersects: Vec<f64>;
    for sphere in spheres{
        intersects = intersect_ray_sphere(&camera_origin, &view, sphere); //two intersections
        if intersects[1].within(tmin, tmax) && intersects[1] < closest_t {
            closest_t = intersects[1];
            closest_sphere = Some(sphere)
        }
        if intersects[0].within(tmin, tmax) && intersects[0] < closest_t {
            closest_t = intersects[0];
        }
    }
    return  (closest_sphere, closest_t);
}
*/

fn intersect_ray_sphere(camera_origin: &Vec<f64>, view: &Vec<f64>, sphere: &Sphere) -> Vec<f64> {
    let radius = sphere.radius;
    let camera_to_center = vec![camera_origin[0] - sphere.center[0], camera_origin[1] - sphere.center[1], camera_origin[2] - sphere.center[2]];
    //let camera_to_center = vec![sphere.center[0] - camera_origin[0], sphere.center[1] - camera_origin[1], sphere.center[2] - camera_origin[2]];
    //println!("{:?}",camera_to_center);
    //ray equation moddeled by point = t*view + camera_origin
    //circle equation moddled by dot_product(point - center, point - center) = r^2
    //sub ray into circle: dot_product((t*view+camera_origin) - center, (t*view+camera_origin) - center) = r^2
    // cam_to center = camorgigin - center
    //plug in cam to center dot_product(camera_to_center + t*view, camera +t*view) = r^2
    //expand dot product: dot_product((cam_to_center + t*view), view) + dot_product((cam_to_center + t*view), t*view)
    //expand again: dot(cam_to_center, cam_to_center) + dot(t*view, cam_to_center) + dot(cam_to_center, t*view) + dot(t*view, t*view)
    //finally: t^2*dot(view,view) + t*2dot(view, am_to_center) + dot(cam_to_center, cam_to_center) - r^2 = 0
    //DO QUADRATIC, SOLVE FOR T
    //println!("{:?}, {:?}", camera_to_center, view);
    let a: f64 = dot_product(&view, &view);
    let b :f64 = 2.0 * dot_product(&view, &camera_to_center);
    let c: f64 = (dot_product(&camera_to_center, &camera_to_center)) - radius.powi(2);

    let discriminant: f64 = b.powi(2)  - (4.0*a*c);
    if discriminant < 0.0 {
        return vec![f64::INFINITY, f64::INFINITY];
    }
    //println!("{:?}", discriminant);
    //then do quadratic formula
    let t1: f64 = (-b + f64::sqrt(discriminant)) / (2.0*a);
    let t2: f64 = (-b - f64::sqrt(discriminant)) / (2.0*a);
    //println!("{}, {}", t1, t2);
    return vec![t1, t2];
}
fn dot_product(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut product: f64 = 0.0;
    for i in 0..a.len(){
        product = product +  (a[i] * b[i]);
    }
    return product;
}

fn canvas_to_viewport(x: f64, y: f64, view_width: f64, view_height: f64, distance: f64, z_rotation_matrix: &Vec<Vec<f64>>) -> Vec<f64> {

    let view_x:f64 = (x*(WIDTH as f64)/view_width) as f64;
    let view_y:f64 = (y*(HEIGHT as f64)/view_height) as f64;
    let mut view = vec![view_x, view_y, distance];

    view = matrix_multiplication(z_rotation_matrix, view);
    //println!("{:?}", view);
    return view;
    
}

fn compute_lighting(intersection: &Vec<f64>, normal: &Vec<f64>, light: &Vec<Light>, to_cam: Vec<f64>, specularity: f64, spheres: &Vec<Sphere>) -> f64 {
    let mut i: f64 = 0.0;
    let mut light_direction: Vec<f64>;
    let mut t_max: f64;
    for light in light {
        if light.typ == "Ambient" {
            i += light.intensity;
        }
        else {
            if light.typ == "Point" {
                light_direction = vec![light.position[0] - intersection[0], light.position[1] - intersection[1], light.position[2] - intersection[2]];
                t_max = 1.0;
            }
            else {
                light_direction = light.direction.clone();
                t_max = f64::INFINITY
            }
            //shadows
            let mut shadow_sphere: Option<&Sphere> = None;
            let mut shadow_t: f64 = f64::INFINITY;
            let mut intersects: Vec<f64>;
            for sphere in spheres{
                intersects = intersect_ray_sphere(&intersection, &light_direction, sphere); //two intersections
                if intersects[0].within(0.001, t_max) && intersects[0] < shadow_t {
                    shadow_t = intersects[0];
                    shadow_sphere = Some(sphere);
                }
                if intersects[1].within(0.001, t_max) && intersects[1] < shadow_t {
                    shadow_t = intersects[1];
                    shadow_sphere = Some(sphere)
                }
            }
            //for diffuse reflection
            if shadow_sphere == None {
                let light_to_cam = dot_product(&normal, &light_direction);
                if light_to_cam > 0.0 {
                    //this is true because of trigonometry
                    //cos = distance to cam / normal + light direction
                    i += (light.intensity * light_to_cam)/((vec3_length(&normal) * vec3_length(&light_direction)))
                }
                //for specular reflection
                if specularity != -1.0 {
                    let reflection =  vec3_negation(&vec3_multiply_by_float(&normal, dot_product(&normal, &light_direction) * 2.0), &light_direction);
                    let reflection_cam_distance = dot_product(&reflection, &to_cam);
                    if reflection_cam_distance > 0.0 {
                        i += light.intensity * f64::powf(reflection_cam_distance/(vec3_length(&reflection)* vec3_length(&to_cam)), specularity)
                    }
                }
            }
            
        }
    }
    return i
}

//vector manipulation functions
fn vec3_addition(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    let mut product: Vec<f64> = vec![0.0,0.0,0.0];
    for i in 0..a.len() {
        product[i] = a[i] + b[i]
    }
    return product;
}
fn vec3_negation(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    let mut product: Vec<f64> = vec![0.0,0.0,0.0];
    for i in 0..a.len() {
        product[i] = a[i] - b[i]
    }
    return product;
}
fn vec3_multiply_by_float(vec3: &Vec<f64>, multiplier: f64) -> Vec<f64> {
    let mut product: Vec<f64> = vec![0.0,0.0,0.0];
    for i in 0..vec3.len() {
        product[i] = multiplier * vec3[i]
    }
    return product
}
fn vec3_divide_by_float(vec3: &Vec<f64>, multiplier: f64) -> Vec<f64> {
    let mut product: Vec<f64> = vec![0.0,0.0,0.0];
    for i in 0..vec3.len() {
        product[i] = vec3[i] / multiplier
    }
    return product

}
fn vec3_length(vec3: &Vec<f64>) -> f64 {
    return f64::sqrt(vec3[0].powi(2) + vec3[1].powi(2)+ vec3[2].powi(2))
}
fn multiply_color_by_float(vec3: &Vec<u8>, multiplier: f64) -> Vec<u8> {
    let mut product: Vec<u8> = vec![0,0,0];
    for i in 0..vec3.len() {
        product[i] = ((multiplier) * vec3[i] as f64).round() as u8
    }
    return product
}
fn matrix_multiplication(transformation: &Vec<Vec<f64>>, matrix: Vec<f64>) -> Vec<f64> {
    let mut new = vec![0.0,0.0,0.0];
    for i in 0..transformation.len() {
        let trans = &transformation[i];
        let product = dot_product(trans, &matrix);
        new[i] = product
    }
    return new
}