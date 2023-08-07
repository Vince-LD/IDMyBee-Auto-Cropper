use argparse::{ArgumentParser, Store, StoreTrue, List };
use opencv::{
    imgcodecs, 
    highgui, 
    imgproc, 
    core::{Vector, Point2f, Mat, Size, DECOMP_LU, BORDER_CONSTANT, Scalar, Rect},
    prelude::*, 
    objdetect::*,
    types::VectorOfPoint2f,
};

use num_derive::FromPrimitive;
use anyhow::{Result, Error};

#[derive(FromPrimitive)]
enum ZoomMode {
    Large = 0,
    Medium = 1,
    Zoom = 2,
}

fn parse_markers(points: &Vector<VectorOfPoint2f>, marker_ids : &Vector<i32>) -> Result<VectorOfPoint2f, opencv::Error> {
    // Crée un nouveau vecteur réorganisé en suivant les indices du vecteur marker_ids

    let mut reordered_points: VectorOfPoint2f = VectorOfPoint2f::from_elem(Point2f::new(0., 0.), 4);
    for (i, new_idx) in marker_ids.iter().enumerate() { 
        let point = points.get(i)?.get(new_idx as usize)?;        
        reordered_points.set(new_idx as usize, point)?;
    }
    Ok(reordered_points)
}

fn resize_if_larger_dims(img : &Mat, out_size : &Size) -> Result<Mat, opencv::Error> {
    // Resize l'image si elle est plus petite que les dimensions d'output
    let img_size = img.size()?;
    let width_ratio : f64 = ((out_size.width / img_size.width) as f64).max(1.);
    let height_ratio : f64 = ((out_size.height / img_size.height) as f64).max(1.);

    let mut resized_img = img.clone();
    println!("Original image size {:?}", img.size()?);
    println!("Image resized by w{} h{}", width_ratio, height_ratio);
    imgproc::resize(
        &img,
        &mut resized_img,
        Size::default(),
        width_ratio, height_ratio,
        imgproc::INTER_LANCZOS4
    ).unwrap();
    
    println!("Resized image size {:?}", resized_img.size()?);
    Ok(resized_img)
}

fn correct_image(img : &Mat, points : &VectorOfPoint2f, out_size: &Size, zoom: &f32) -> Result<Mat, opencv::Error> {

    let h = out_size.height as f32;
    let w = out_size.width as f32;
    let top_y = -0.5 * h * (zoom - 1.);
    let bot_y = h + 0.5 * h * (zoom - 1.);
    let right_x = w * zoom;

    let target_points: VectorOfPoint2f = Vector::from_slice(
        &[
            Point2f::new(0.0, top_y),
            Point2f::new(right_x, top_y),
            Point2f::new(right_x, bot_y),
            Point2f::new(0.0, bot_y),
        ]
    );

    // Obtenir la matrice de transformation en perspective
    let perspective_transform = imgproc::get_perspective_transform(&points, &target_points, DECOMP_LU)?; 

    // Créer une nouvelle matrice pour stocker l'image transformée
    let mut transformed_image = Mat::default();

    // Appliquer la transformation en perspective à l'image
    imgproc::warp_perspective(
        &img,
        &mut transformed_image,
        &perspective_transform,
        img.size()?,
        imgproc::INTER_LANCZOS4,
        BORDER_CONSTANT,
        Scalar::default(),
    )?;

    Ok(transformed_image)
}

fn show_image(image : &Mat) -> Result<(), opencv::Error> {
    highgui::imshow("Preprocess image", image)?;
    highgui::wait_key(0)?;
    Ok(())
}

fn main() -> Result<()>{
    // let mut verbose = false;
    let mut input_path = String::new();
    // let mut opt_output_path: Option<String> = None;
    let mut output_paths: Vec<String> = vec![];
    let mut out_dim = vec![600, 400];
    let mut show = false;
    let mut zoom_vec : Vec<f32> = vec![1.];

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("This tools is used to preprcocess photos taken with the ID My Bee protocol. It automatically crop and correct the photo angle.");

        parser.refer(&mut input_path)
            .add_option(&["-i", "--img"], Store,
            "Input image path to preprocess.")
            .required();
        
        parser.refer(&mut output_paths)
            .add_option(&["-o", "--img_out"], List,
            "Output preprocessed image path.  /!\\ The number of output files given must be 0 or the same as the number of zoom levels. If not given, the default output file(s) will follow the pattern: '[input_folder]/[base input filename]_preproc_z[zoom level].[input file extension]'");

        parser.refer(&mut out_dim)
            .add_option(&["-d", "--out_dim"], List,
            "Output image dimensions width height (e.g. default is '-d 600 400').");

        parser.refer(&mut zoom_vec)
            .add_option(&["-z", "--zoom"], List,
            "The zoom to apply. Multiple values can be included. Float numbers can be used. Default is 1.");
        
        parser.refer(&mut show)
            .add_option(&["-s", "--show"], StoreTrue,
            "Show the image in a window instead of saving it. Once the windows is open, press any key to exit, Ctrl-C to copy the image and Ctrl-S to save it manually.");
            
        parser.parse_args_or_exit();
    }

    println!("Input path: {input_path:?}");
    
    // let mut output_paths: Vec<String> = vec![];
    if !output_paths.is_empty() && output_paths.len() != zoom_vec.len() {
        return Err(
            anyhow::anyhow!(
                "Mismatch between the number of output paths (={:?}) and the number of zoom values (={:?})", 
                output_paths.len(), 
                zoom_vec.len()
            )
        )
    }
    
    for (i, &zoom) in zoom_vec.iter().enumerate() {
        match output_paths.get(i) {
            Some(_) => (),
            None => {
                let (base_path, extenstion) = input_path.rsplit_once('.').ok_or(format!("Input file {input_path:?}")).map_err(Error::msg)?;

                let out_path = format!("{base_path}_preproc_z{:.2}.{extenstion}", zoom).replacen(".", "-", 1);
                println!("{base_path}_preproc_z{:.3}.{extenstion}", zoom);
                println!("Output path was not specified so image will be written to {out_path}");
                output_paths.push(out_path);
                ()
            }
        }
    };
    println!("Output path: {output_paths:?}");

    // let img = get_image(&input_path).to_rgba8();    
    let img = imgcodecs::imread(&input_path, imgcodecs::IMREAD_UNCHANGED)?;
    let out_size = Size::new(out_dim[0], out_dim[1]);
    let img = resize_if_larger_dims(&img, &out_size)?;
    // show_image(&img);
    let mut gray_image = Mat::default();
    imgproc::cvt_color(&img, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;
    // show_image(&gray_image);
    let aruco_detector = ArucoDetector::new(  
        &get_predefined_dictionary(PredefinedDictionaryType::DICT_4X4_50)?,
        &DetectorParameters::default().unwrap(),
        RefineParameters::new(10., 3., true).unwrap(),
    ).unwrap();
    let mut markers_coor : Vector<VectorOfPoint2f> = Vector::new();
    let mut marker_ids: Vector<i32> = Vector::new();
    let mut rejected_img_points: Vector<VectorOfPoint2f> = Vector::new();
    aruco_detector.detect_markers(&gray_image, &mut markers_coor, &mut marker_ids, &mut rejected_img_points)?;
    println!("Markers found: {:?}", marker_ids);
    
    for (zoom, out_path) in zoom_vec.iter().zip(output_paths.iter()) {
        
        if markers_coor.len() == 4 {
            // println!("{:?}", markers_coor);
            let ordered_points = parse_markers(&markers_coor, &marker_ids)?;
    
            // let ordered_points = parse(&marker_points, &marker_ids);
            println!("Points used from marker 0 to 3: {:?}", ordered_points);
    
            
            let warped_image = correct_image(&img, &ordered_points, &out_size, &zoom)?;

            let final_image = Mat::roi(&warped_image, Rect {
                x: 0,
                y: 0,
                width: out_size.width,
                height: out_size.height,
            }).unwrap();
    
            if show {
                show_image(&final_image)?;
            } else {
                println!("Saving image to {:?}", out_path);
                imgcodecs::imwrite(&out_path, &final_image, &Vector::new())?;
    
            }

        } else {
            return Err(anyhow::anyhow!("Error: {:?} markers were found instead of 4.", markers_coor.len()))
        }

    };
    return Ok(())
}