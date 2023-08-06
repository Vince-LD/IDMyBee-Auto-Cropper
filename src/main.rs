// use image::{Rgba, imageops, DynamicImage};
// use imageproc::geometric_transformations::{Projection, Interpolation, warp};
use argparse::{ArgumentParser, Store, StoreOption, StoreTrue, List };
use opencv::{
    imgcodecs, 
    highgui, 
    imgproc, 
    core::{Vector, Point2f, Mat, Size, DECOMP_LU, BORDER_CONSTANT, Scalar, Rect},
    prelude::*, 
    objdetect::*,
    types::VectorOfPoint2f
};

fn parse_markers(points: &Vector<VectorOfPoint2f>, marker_ids : &Vector<i32>) -> VectorOfPoint2f {
    // Crée un nouveau vecteur réorganisé en suivant les indices du vecteur marker_ids

    let mut reordered_points: VectorOfPoint2f = VectorOfPoint2f::from_elem(Point2f::new(0., 0.), 4);
    for (i, new_idx) in marker_ids.iter().enumerate() { 
        let point_vec = points.get(i as usize).unwrap();
        let point = point_vec.get(new_idx as usize).unwrap();
        reordered_points.set(new_idx as usize, point).unwrap();
    }
    reordered_points
}

fn resize_if_larger_dims(img : &Mat, dims : &Size) -> Mat {
    // Resize l'image si elle est plus petite que les dimensions d'output
    let img_size = img.size().unwrap();
    let width_ratio : f64 = (dims.width as f64 / img_size.width as f64).max(1.);
    let height_ratio : f64 = (dims.height as f64 / img_size.height as f64).max(1.);

    let mut resized_img = img.clone();
    println!("Original image size {:?}", img.size().unwrap());
    println!("Image resized by w{} h{}", width_ratio, height_ratio);
    imgproc::resize(
        &img,
        &mut resized_img,
        Size::default(),
        width_ratio, height_ratio,
        imgproc::INTER_LANCZOS4
    ).unwrap();
    
    println!("Resized image size {:?}", resized_img.size().unwrap());
    resized_img
}

fn correct_image(img : &Mat, points : &VectorOfPoint2f, out_size: &Size) -> Mat {
    let target_points: VectorOfPoint2f = Vector::from_slice(
        &[
            Point2f::new(0.0, 0.0),
            Point2f::new(out_size.width as f32, 0.0),
            Point2f::new(out_size.width as f32, out_size.height as f32),
            Point2f::new(0.0, out_size.height as f32),
        ]
    );

    // Obtenir la matrice de transformation en perspective
    let perspective_transform = imgproc::get_perspective_transform(&points, &target_points, DECOMP_LU).unwrap(); 

    // Créer une nouvelle matrice pour stocker l'image transformée
    let mut transformed_image = Mat::default();

    // Appliquer la transformation en perspective à l'image
    imgproc::warp_perspective(
        &img,
        &mut transformed_image,
        &perspective_transform,
        img.size().unwrap(),
        imgproc::INTER_LANCZOS4,
        BORDER_CONSTANT,
        Scalar::default(),
    )
    .unwrap();

    transformed_image
}

fn show_image(image : &Mat) {
    highgui::imshow("Preprocess image", image).unwrap();
    highgui::wait_key(0).unwrap();
}

fn main() {
    // let mut verbose = false;
    let mut input_path = String::new();
    let mut opt_output_path: Option<String> = None;
    let mut out_dim = vec![600, 300];
    let mut show = false;
    

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("This tools is used to preprcocess photos taken with the Id My Bee protocol. It automatically crop and correct the photo angle.");
        
        // parser.refer(&mut verbose)
        //     .add_option(&["-v", "--verbose"], StoreTrue,
        //     "Activate verbose");
        
        parser.refer(&mut input_path)
            .add_option(&["-i", "--img"], Store,
            "Input image path to preprocess.")
            .required();
        
        parser.refer(&mut opt_output_path)
            .add_option(&["-o", "--img_out"], StoreOption,
            "Output preprocessed image path.");

        parser.refer(&mut out_dim)
            .add_option(&["-d", "--out_dim"], List,
            "Output image dimensions width height (e.g. default is '-d 600 300').");
        
            
        parser.refer(&mut show)
        .add_option(&["-s", "--show"], StoreTrue,
        "Show the image in a window instead of saving it. Once the windows is open, press 'q' to quit.");
        
        parser.parse_args_or_exit();
    }

    println!("Input path: {input_path:?}");
    
    let mut output_path = String::new(); 
    if !show {
        output_path = match opt_output_path {
            Some(path) => path,
            None => {
                let mut out_path = input_path.clone();
                let (base_path, extenstion) = out_path.rsplit_once('.').unwrap();
                out_path = format!("{base_path}_preproc.{extenstion}");
                println!("Output path was not specified so image will be written to {out_path}");
                out_path
            },
        };
        println!("Output path: {output_path:?}");
    }
    
    // let img = get_image(&input_path).to_rgba8();    
    let img = imgcodecs::imread(&input_path, imgcodecs::IMREAD_UNCHANGED).unwrap();
    let out_size = Size::new(out_dim[0], out_dim[1]);
    let img = resize_if_larger_dims(&img, &out_size);
    // show_image(&img);
    let mut gray_image = Mat::default();
    imgproc::cvt_color(&img, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    // show_image(&gray_image);

    let aruco_detector = ArucoDetector::new(  
        &get_predefined_dictionary(PredefinedDictionaryType::DICT_4X4_50).unwrap(),
        &DetectorParameters::default().unwrap(),
        RefineParameters::new(10., 3., true).unwrap(),
    ).unwrap();

    let mut markers_coor : Vector<VectorOfPoint2f> = Vector::new();
    let mut marker_ids: Vector<i32> = Vector::new();
    let mut rejected_img_points: Vector<VectorOfPoint2f> = Vector::new();
    aruco_detector.detect_markers(&gray_image, &mut markers_coor, &mut marker_ids, &mut rejected_img_points).unwrap();
    
    println!("Markers found: {:?}", marker_ids);

    if markers_coor.len() == 4 {
        // println!("{:?}", markers_coor);
        let ordered_points = parse_markers(&markers_coor, &marker_ids);

        // let ordered_points = parse(&marker_points, &marker_ids);
        println!("Points used from marker 0 to 3: {:?}", ordered_points);

        
        let warped_image = correct_image(&img, &ordered_points, &out_size);
        
        // let final_image = match Mat::roi(&warped_image, Rect {
        //     x: 0,
        //     y: 0,
        //     width: out_size.width,
        //     height: out_size.height,
        // }){
        //     Ok(img) => img,
        //     Err(err) => {
        //         println!("{:?}", err);
        //         warped_image
        //     }
        // };
        let final_image = Mat::roi(&warped_image, Rect {
            x: 0,
            y: 0,
            width: out_size.width,
            height: out_size.height,
        }).unwrap();

        if show {
            show_image(&final_image)
        } else {
            println!("Saving image to {:?}", output_path);
            imgcodecs::imwrite(&output_path, &final_image, &Vector::new()).unwrap();

        }

    } else {
        println!("Error: {:?} markers were found instead of 4.", markers_coor.len())
    }
}
