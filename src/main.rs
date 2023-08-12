use argparse::{ArgumentParser, Store, StoreTrue, List };
use anyhow::{Error, Result};
use opencv::{
    core::{Mat, Point2f, Rect, Vector, Size},
    imgcodecs,
    types::VectorOfPoint2f,
};

mod marker_utils;
use marker_utils::marker_processing::*;

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
            "The zoom to apply (can be float numbers). Multiple values can be used. Default is 1.");
        
        parser.refer(&mut show)
            .add_option(&["-s", "--show"], StoreTrue,
            "Show the image in a window instead of saving it. Once the windows is open, press any key to exit, Ctrl-C to copy the image and Ctrl-S to save it manually.");
            
        parser.parse_args_or_exit();
    }

    println!("Input path: {input_path:?}");
    
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

                let out_path = format!("{base_path}_preproc_z{:.2}.{extenstion}", zoom).replacen('.', "-", 1);
                println!("{base_path}_preproc_z{:.3}.{extenstion}", zoom);
                println!("Output path was not specified so image will be written to {out_path}");
                output_paths.push(out_path);
            }
        }
    };
    println!("Output path: {output_paths:?}");

    // let img = get_image(&input_path).to_rgba8();    
    let mut img = imgcodecs::imread(&input_path, imgcodecs::IMREAD_UNCHANGED)?;
    let out_size = Size::new(out_dim[0], out_dim[1]);
    img = resize_if_larger_dims(img, &out_size)?;
    // show_image(&img);

    let (markers_coor, markers_id, rejected_markers) = get_image_markers(&img)?;
    let ordered_points = parse_markers(&markers_coor, &markers_id)?;
    
    if markers_coor.len() == 4 {
        println!("Points used from marker #0 to #3: {:?}", ordered_points);
        for (zoom, out_path) in zoom_vec.iter().zip(output_paths.iter()) {
        
            // println!("{:?}", markers_coor);
    
            // let ordered_points = parse(&marker_points, &markers_id);
    
            
            let warped_image = correct_image(&img, &ordered_points, &out_size, zoom)?;

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
                imgcodecs::imwrite(out_path, &final_image, &Vector::new())?;
    
            }
        }
    }  else {
        let rejected_marker_positions : VectorOfPoint2f = rejected_markers.iter().map(
            |p_vec| p_vec.iter().fold(
                Point2f::default(), |sum_p, p| sum_p + p
            ) / 4.
        ).collect();
        return Err(anyhow::anyhow!("Error: {:?} markers were found instead of 4.\nFollowing markers were rejected: {:?}\nThe image may be too blurred or there may be stray reflections on the markers.", 
            markers_coor.len(), rejected_marker_positions
        ))
    };

    Ok(())
}