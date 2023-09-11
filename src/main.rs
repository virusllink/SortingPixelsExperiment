use std::{cmp::Reverse, fs};
use std::fs::File;
use std::io::Write;
use std::path::{Path, MAIN_SEPARATOR_STR};
use image::{Rgba, DynamicImage, GenericImageView, GenericImage};

fn main() {
    // Check if the settings file exists
    if File::open("settings.txt").is_err() {
        // If it doesn't exist, create it with default settings
        let mut file = File::create("settings.txt").unwrap();
        let default_settings_string = "input //The path where film/image(s) are, as well as where the program will output the result\nleft //The sort direction (Possible values:left,right,down,up)\nred //What value to sort by (Possible values:red,green,blue,hue,saturation,value)\n0.5 //The lower bound of values (Range: 0.0-1.0) (Anything more than this will get sorted)\n1.0 //The upper bound of values (Range: 0.0-1.0) (Anything less than this will get sorted)\nred //What value should be used to create the contrast map (Possible values:red,green,blue,hue,saturation,value)\nfalse //Should the program print debug messages and create debug images? (Either true or false)";
        file.write_all(default_settings_string.as_bytes()).unwrap();
    }

    // Load the settings from the settings file
    let lines = fs::read_to_string("settings.txt").unwrap().lines().map(|x| x.split("//").next().unwrap().trim().to_string().to_ascii_lowercase()).collect::<Vec<String>>();

    // Error check the settings file
    if lines.len() != 7 {
        println!("The settings file is not formatted correctly or has the wrong amount of lines. Please make sure the number of lines inside the file is equal to 7 (No empty lines at the end of the file). Please delete it and run the program again to create a new one.");
        return;
    } else if lines[6] == "true" {
        println!("{:?}, {:?}", lines, !Path::new(&lines[0]).exists() || !Path::new(&lines[0]).is_dir());
    } else if !Path::new(&lines[0]).exists() || !Path::new(&lines[0]).is_dir() {
        println!("The input path is not a directory or does not exist. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[1] != "left" && lines[1] != "right" && lines[1] != "down" && lines[1] != "up" {
        println!("The sort direction is not valid. Please make sure the value is supported and spelt correctly. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[2] != "red" && lines[2] != "green" && lines[2] != "blue" && lines[2] != "hue" && lines[2] != "saturation" && lines[2] != "value" {
        println!("The sort by value is not valid. Please make sure the value is supported and spelt correctly. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[3].parse::<f32>().is_err() || lines[3].parse::<f32>().unwrap() < 0.0 || lines[3].parse::<f32>().unwrap() > 1.0 {
        println!("The contrast map lower bound is not valid. Please make sure the value is a number between 0.0 and 1.0. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[4].parse::<f32>().is_err() || lines[4].parse::<f32>().unwrap() < 0.0 || lines[4].parse::<f32>().unwrap() > 1.0 {
        println!("The contrast map upper bound is not valid. Please make sure the value is a number between 0.0 and 1.0. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[5] != "red" && lines[5] != "green" && lines[5] != "blue" && lines[5] != "hue" && lines[5] != "saturation" && lines[5] != "value" {
        println!("The contrast type is not valid. Please make sure the value is supported and spelt correctly. Please delete it and run the program again to create a new one.");
        return;
    } else if lines[6] != "true" && lines[6] != "false" {
        println!("The debug value is not valid. Please make sure the value is either true or false. Please delete it and run the program again to create a new one.");
        return;
    }

    // Initialise the program settings and start the program
    let program_settings = ProgramSettings {
        input_path: lines[0].clone(),
        sort_direction: match lines[1].as_str() {
            "left" => SortDirection::Left,
            "right" => SortDirection::Right,
            "down" => SortDirection::Down,
            "up" => SortDirection::Up,
            _ => SortDirection::Left, 
        },
        sort_by: match lines[2].as_str() {
            "red" => SortBy::Red,
            "green" => SortBy::Green,
            "blue" => SortBy::Blue,
            "hue" => SortBy::Hue,
            "saturation" => SortBy::Saturation,
            "value" => SortBy::Value,
            _ => SortBy::Red,
        },
        contrast_map_lower: 0.5,
        contrast_map_upper: 1.0,
        contrast_type: match lines[5].as_str() {
            "red" => ContrastType::Red,
            "green" => ContrastType::Green,
            "blue" => ContrastType::Blue,
            "hue" => ContrastType::Hue,
            "saturation" => ContrastType::Saturation,
            "value" => ContrastType::Value,
            _ => ContrastType::Red,
        },
        should_debug: lines[6].parse().unwrap(),
    };
    manage_sort(&program_settings);
}

fn manage_sort(program_settings: &ProgramSettings) {
    let input_path = Path::new(&program_settings.input_path).canonicalize().unwrap();
    let binding = input_path.clone().to_str().unwrap().to_string() + MAIN_SEPARATOR_STR + "out";
    let output_path = Path::new(&binding);
 
    if !output_path.exists() {
        let result = fs::create_dir(output_path);
        if result.is_err() {
            println!("Unable to create the output directory {:?}. {:?}", output_path.canonicalize().unwrap_err().kind(), result.err().unwrap());
            return;
        }
    }

    let paths = fs::read_dir(input_path).unwrap();
    for path in paths {
        let unwrap_path = path.unwrap();
        if !unwrap_path.path().is_dir() {
            let path_string = unwrap_path.path().to_str().unwrap().to_string();
            let output_path_and_name = output_path.to_str().unwrap().to_string().clone() + MAIN_SEPARATOR_STR + unwrap_path.path().file_name().unwrap().to_str().unwrap();
            start_sort(program_settings, &path_string, &output_path_and_name);
        }
    }
}

fn start_sort(program_settings: &ProgramSettings, input_image_path: &String, output_path_and_name: &String) {
    // Open the image and get the pixels
    println!("Opening image: {}", input_image_path);
    let img: DynamicImage = image::open(input_image_path).unwrap();
    let mut pixels_vec: Vec<Rgba<u8>> = img.pixels().map(|pixel| pixel.2).collect();

    // Create a contrast map from the pixels
    println!("Creating contrast map");
    let mut contrast_map: Vec<bool> = Vec::with_capacity(pixels_vec.len());
    create_contrast_map(program_settings, &pixels_vec, &mut contrast_map);

    //Save the contrast map for debugging
    if program_settings.should_debug {
        println!("Saving contrast map");
        let mut contrast_map_img: DynamicImage = DynamicImage::new_rgba8(img.width(), img.height());
        contrast_map.iter().enumerate().for_each(|(i, pixel)| contrast_map_img.put_pixel((i%img.width() as usize) as u32, (i/img.width() as usize) as u32, if *pixel {Rgba([255, 255, 255, 255])} else {Rgba([0, 0, 0, 255])}));
        contrast_map_img.save(output_path_and_name.clone() + "mask.png").unwrap();
    }

    // Sort the pixels
    println!("Sorting pixels");
    sort_pixels(program_settings, &mut pixels_vec, &contrast_map, img.width() as usize, img.height() as usize);

    // Create a new image from the pixels
    println!("Creating new image");
    let mut new_img: DynamicImage = DynamicImage::new_rgba8(img.width(), img.height());
    pixels_vec.iter().enumerate().for_each(|(i, pixel)| new_img.put_pixel((i%img.width() as usize) as u32, (i/img.width() as usize) as u32, pixel.clone()));
    new_img.save(output_path_and_name).unwrap();

    fn create_contrast_map(program_settings: &ProgramSettings, pixels_vec: &Vec<Rgba<u8>>, contrast_map: &mut Vec<bool>) {
        match program_settings.contrast_type {
            ContrastType::Red => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push((pixel[0] as f32/255.0) >= program_settings.contrast_map_lower && (pixel[0] as f32/255.0) <= program_settings.contrast_map_upper));
            },
            ContrastType::Green => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push((pixel[1] as f32/255.0) >= program_settings.contrast_map_lower && (pixel[1] as f32/255.0) <= program_settings.contrast_map_upper));
            },
            ContrastType::Blue => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push((pixel[2] as f32/255.0) >= program_settings.contrast_map_lower && (pixel[2] as f32/255.0) <= program_settings.contrast_map_upper));
            },
            ContrastType::Hue => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push(rgb_to_hsv(&pixel).0/360.0 >= program_settings.contrast_map_lower && rgb_to_hsv(&pixel).0/360.0 <= program_settings.contrast_map_upper));
            },
            ContrastType::Saturation => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push(rgb_to_hsv(&pixel).1/100.0 >= program_settings.contrast_map_lower && rgb_to_hsv(&pixel).1/100.0 <= program_settings.contrast_map_upper));
            },
            ContrastType::Value => {
                pixels_vec.iter().for_each(|pixel| contrast_map.push(rgb_to_hsv(&pixel).2/100.0 >= program_settings.contrast_map_lower && rgb_to_hsv(&pixel).2/100.0 <= program_settings.contrast_map_upper));
            },
        }
    }

    fn sort_pixels(program_settings: &ProgramSettings, pixels_vec: &mut Vec<Rgba<u8>>, contrast_map: &Vec<bool>, width: usize, height: usize) {
        match program_settings.sort_by {
            SortBy::Red => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[0]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| pixel[0]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[0]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| pixel[0]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
            SortBy::Green => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[1]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| pixel[1]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[1]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| pixel[1]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
            SortBy::Blue => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[2]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by_key(|pixel| pixel[2]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| Reverse(pixel[2]));
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by_key(|pixel| pixel[2]);
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
            SortBy::Hue => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).0.partial_cmp(&rgb_to_hsv(b).0).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).0.partial_cmp(&rgb_to_hsv(b).0).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).0.partial_cmp(&rgb_to_hsv(b).0).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).0.partial_cmp(&rgb_to_hsv(b).0).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
            SortBy::Saturation => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).1.partial_cmp(&rgb_to_hsv(b).1).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).1.partial_cmp(&rgb_to_hsv(b).1).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).1.partial_cmp(&rgb_to_hsv(b).1).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).1.partial_cmp(&rgb_to_hsv(b).1).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
            SortBy::Value => {
                match program_settings.sort_direction {
                    SortDirection::Left => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).2.partial_cmp(&rgb_to_hsv(b).2).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Right => {
                        for y in 0..height {
                            // Get the row of pixels
                            let mut row = pixels_vec[y*width..(y+1)*width].to_vec();
                            // Sort the row only where spans of the contrast map are true
                            let mut i = 0;
                            while i < row.len() {
                                if contrast_map[y*width+i] {
                                    let mut j = i+1;
                                    while j < row.len() && contrast_map[y*width+j] {
                                        j += 1;
                                    }
                                    row[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).2.partial_cmp(&rgb_to_hsv(b).2).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the row back into the pixels vector
                            pixels_vec[y*width..(y+1)*width].copy_from_slice(&row); 

                            if program_settings.should_debug {
                                println!("{}%", (y as f32/height as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Up => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).2.partial_cmp(&rgb_to_hsv(b).2).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                    SortDirection::Down => {
                        for x in 0..width {
                            // Get the column of pixels
                            let mut column: Vec<Rgba<u8>> = Vec::with_capacity(height);
                            for y in 0..height {
                                column.push(pixels_vec[y*width+x]);
                            }
                            // Sort the column only where spans of the contrast map are true
                            let mut i = 0;
                            while i < column.len() {
                                if contrast_map[i*width+x] {
                                    let mut j = i+1;
                                    while j < column.len() && contrast_map[j*width+x] {
                                        j += 1;
                                    }
                                    column[i..j].sort_unstable_by(|a, b| rgb_to_hsv(a).2.partial_cmp(&rgb_to_hsv(b).2).unwrap());
                                    i = j;
                                } else {
                                    i += 1;
                                }
                            }
                            // Put the column back into the pixels vector
                            for y in 0..height {
                                pixels_vec[y*width+x] = column[y];
                            }

                            if program_settings.should_debug {
                                println!("{}%", (x as f32/width as f32)*100.0);
                            }
                        }
                    },
                }
            },
        }
    }
}

fn rgb_to_hsv(colour: &Rgba<u8>) -> (f32, f32, f32) {
    let r: f32 = colour[0] as f32/255.0;
    let g: f32 = colour[1] as f32/255.0;
    let b: f32 = colour[2] as f32/255.0;

    let max: f32 = r.max(g).max(b);
    let min: f32 = r.min(g).min(b);
    let delta: f32 = max - min;

    let mut h: f32 = 0.0;
    if max == min {
        h = 0.0;
    } else if max == r {
        h = (60.0 * ((g - b) / delta) + 360.0) % 360.0;
    } else if max == g {
        h = (60.0 * ((b - r) / delta) + 120.0) % 360.0;
    } else if max == b {
        h = (60.0 * ((r - g) / delta) + 240.0) % 360.0;
    }

    let s: f32;
    if max == 0.0 {
        s = 0.0;
    } else {
        s = (delta / max) * 100.0;
    }

    return (h, s, max * 100.0);
}

struct ProgramSettings {
    input_path: String,
    sort_direction: SortDirection,
    sort_by: SortBy,
    contrast_map_lower: f32,
    contrast_map_upper: f32,
    contrast_type: ContrastType,
    should_debug: bool,
}

enum SortDirection {
    Left,
    Right,
    Up,
    Down
}

enum SortBy {
    Red,
    Green,
    Blue,
    Hue,
    Saturation,
    Value,
}

enum ContrastType {
    Red,
    Green,
    Blue,
    Hue,
    Saturation,
    Value,
}
