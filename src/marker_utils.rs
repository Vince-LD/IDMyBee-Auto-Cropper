pub mod marker_processing {
    use opencv::{
        core::{Mat, Point2f, Scalar, Size, Vector, BORDER_CONSTANT, DECOMP_LU},
        highgui, imgproc,
        objdetect::*,
        prelude::*,
        types::VectorOfPoint2f,
    };

    use num_derive::FromPrimitive;

    #[derive(FromPrimitive)]
    enum ZoomMode {
        Large = 0,
        Medium = 1,
        Zoom = 2,
    }

    type MarkersVec = Vector<VectorOfPoint2f>;

    pub fn get_image_markers(
        img: &Mat,
    ) -> Result<(MarkersVec, Vector<i32>, MarkersVec), opencv::Error> {
        let mut gray_image = Mat::default();
        imgproc::cvt_color(&img, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;
        // show_image(&gray_image);
        let aruco_detector = ArucoDetector::new(
            &get_predefined_dictionary(PredefinedDictionaryType::DICT_4X4_50)?,
            &DetectorParameters::default().unwrap(),
            RefineParameters::new(10., 3., true)?,
        )?;
        let mut markers_coor: MarkersVec = Vector::new();
        let mut markers_id: Vector<i32> = Vector::new();
        let mut rejected_markers: MarkersVec = Vector::new();
        aruco_detector.detect_markers(
            &gray_image,
            &mut markers_coor,
            &mut markers_id,
            &mut rejected_markers,
        )?;
        println!("Markers found: {:?}", markers_id);

        Ok((markers_coor, markers_id, rejected_markers))
    }

    pub fn parse_markers(
        points: &MarkersVec,
        markers_id: &Vector<i32>,
    ) -> Result<VectorOfPoint2f, opencv::Error> {
        // Crée un nouveau vecteur réorganisé en suivant les indices du vecteur markers_id

        let mut reordered_points: VectorOfPoint2f =
            VectorOfPoint2f::from_elem(Point2f::new(0., 0.), 4);
        for (i, new_idx) in markers_id.iter().enumerate() {
            let point = points.get(i)?.get(new_idx as usize)?;
            reordered_points.set(new_idx as usize, point)?;
        }
        Ok(reordered_points)
    }

    pub fn resize_if_larger_dims(img: Mat, out_size: &Size) -> Result<Mat, opencv::Error> {
        // Resize l'image si elle est plus petite que les dimensions d'output
        let img_size = img.size()?;
        let width_ratio: f64 = ((out_size.width / img_size.width) as f64).max(1.);
        let height_ratio: f64 = ((out_size.height / img_size.height) as f64).max(1.);

        if width_ratio == 1. && height_ratio == 1. {
            return Ok(img);
        }
        let mut resized_img = img.clone();
        println!("Original image size {:?}", img.size()?);
        println!("Image resized by w{:.2} h{:.2}", width_ratio, height_ratio);
        imgproc::resize(
            &img,
            &mut resized_img,
            Size::default(),
            width_ratio,
            height_ratio,
            imgproc::INTER_LANCZOS4,
        )?;

        println!("Resized image size {:?}", resized_img.size()?);
        Ok(resized_img)
    }

    pub fn correct_image(
        img: &Mat,
        points: &VectorOfPoint2f,
        out_size: &Size,
        zoom: &f32,
    ) -> Result<Mat, opencv::Error> {
        let h = out_size.height as f32;
        let w = out_size.width as f32;
        let top_y = -0.5 * h * (zoom - 1.);
        let bot_y = h + 0.5 * h * (zoom - 1.);
        let right_x = w * zoom;

        let target_points: VectorOfPoint2f = Vector::from_slice(&[
            Point2f::new(0.0, top_y),
            Point2f::new(right_x, top_y),
            Point2f::new(right_x, bot_y),
            Point2f::new(0.0, bot_y),
        ]);

        // Obtenir la matrice de transformation en perspective
        let perspective_transform =
            imgproc::get_perspective_transform(&points, &target_points, DECOMP_LU)?;

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

    pub fn show_image(image: &Mat) -> Result<(), opencv::Error> {
        highgui::imshow("Preprocess image", image)?;
        highgui::wait_key(0)?;
        Ok(())
    }
}
