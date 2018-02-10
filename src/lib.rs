extern crate image;
extern crate num_cpus;
extern crate rand;
extern crate threadpool;

mod core;
mod model;
mod scanline;
mod shape;
mod state;
mod util;
mod worker;

pub use shape::ShapeType;

use std::io::Write;
use std::fs::File;

use model::Model;

const SIZE: usize = 256;

pub fn run(config: Config) {
//    println!("{:?}", config);

    let img = util::load_image(config.in_path.as_ref()).expect("couldn't load image");
    let cpus = num_cpus::get_physical();
    let mut model = Model::new(img, cpus, config.out_size);
    for _ in 0..config.num_shapes {
        model.step(config.shape_type, config.alpha, 1000, config.m);
    }
    if config.out_path.ends_with(".svg") {
        let mut file = File::create(&config.out_path).unwrap();
        file.write_all(model.svg().as_bytes()).unwrap();
    } else {
        model.save_rasterized(&config.out_path).expect("wtf");
    }
}

#[derive(Debug)]
pub struct Config {
    pub in_path: String,
    pub out_path: String,
    pub num_shapes: u32,
    pub shape_type: ShapeType,
    pub out_size: usize,
    pub alpha: u8,
    pub m: u8,
}

#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString, JValue};
    use self::jni::sys::{jstring, jint, jobject};

    static mut CONFIG_OPT: Option<Config> = None;
    static mut MODEL_OPT: Option<Model> = None;

    #[no_mangle]
    pub unsafe extern fn Java_com_github_larryng_primage_jni_Primg_jniInit(
        env: JNIEnv, _: JClass, img_path: JString, shape_type: jint, m: jint) -> jobject {

        let in_path: String = env.get_string(img_path).expect("wtf").into();
        let out_path = String::from("");
        let shape_type = match shape_type {
            0 => ShapeType::Triangle,
            1 => ShapeType::Ellipse,
            2 => ShapeType::Rectangle,
            3 => ShapeType::RotatedRectangle,
            _ => unreachable!(),
        };
        let out_size = 512;
        let alpha = 128;
        let num_shapes = 42;
        let m = m as u8;
        let config = Config {
            in_path,
            out_path,
            num_shapes,
            shape_type,
            out_size,
            alpha,
            m
        };

        let img = util::load_image(config.in_path.as_ref()).expect("couldn't load image");
        let img = util::scaled_to_area(img, SIZE * SIZE);
        let cpus = num_cpus::get_physical();

        let model = Model::new(img, cpus, config.out_size);

        let class = env.find_class("com/github/larryng/primage/jni/PrimgInitResult").expect("couldn't load class");
        let constructor = env.get_method_id(class, "<init>", "(Ljava/lang/Object;III)V").expect("couldn't get constructor");
        let debug: String = format!("cpus: get={}, physical={}", num_cpus::get(), num_cpus::get_physical());
        let debug = JValue::Object(env.new_string(debug).unwrap().into());
        let w = JValue::Int(model.w as i32);
        let h = JValue::Int(model.h as i32);
        let color = JValue::Int(model.bg.to_argb_i32());
        let args = &[debug, w, h, color];
        let obj = env.new_object_by_id(class, constructor, &args[..]).expect("couldn't make PrimgInitResult").into_inner();

        MODEL_OPT = Some(model);
        CONFIG_OPT = Some(config);

        obj
    }

    #[no_mangle]
    pub unsafe extern fn Java_com_github_larryng_primage_jni_Primg_jniStep(
        env: JNIEnv, _: JClass) -> jstring {

        let config = match CONFIG_OPT {
            Some(ref c) => c,
            None => unreachable!(),
        };

        let model = match MODEL_OPT {
            Some(ref mut m) => m,
            None => unreachable!(),
        };

        let (shape, color) = model.step(config.shape_type, config.alpha, 1000, config.m);

        let s = format!("{}:{}", shape.serialize(), color.to_argb_i32());

        env.new_string(s).unwrap().into_inner()
    }
}
