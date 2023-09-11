# Sorting Pixels Experiment
An experiment in image manipulation in rust. 

## How to use
Run the program once; this should create a settings.txt and /input directory where the executable is. Place all images in the input folder (movies should hopefully be supported soon with the use of ffmpeg, but for now you can just split the film into frames and join them afterwards) and change the settings.txt to your liking, and then run the program. The program should then generate an /out folder inside the /input folder, and all images will be saved in there.

## Settings
The settings.txt file should generate with 7 lines of settings:
1. The path to where the images are stored, this can be a relative path (example/images or images (Note how there is no separator at the start of the path)) or an absolute path (C:/example/images)
2. The sort direction. This is the direction the sort is applied e.g. "left" will sort the pixels with the highest value to the left of the image. Possible values: left, right, up, down.
3. What value to use for the sort. E.g. "red" will use the red value of each pixel and sort based off that. Possible values: red, green, blue, hue, saturation, value.
4. The lower bound to create the contrast map with (See How it works to learn more about the contrast map). Possible values: Anywhere from 0.0 to 1.0 (Up to 7(?) decimal places)
5. The upper bound to create the contrast map. Same as before. (Hint, the application doesn't enforce that upper bound > lower bound, meaning that you can have a lower bound of 1.0 and an upper bound of 0.5 which can create different results, especially when using "hue" as the value to create the contrast map with)
6. What value should be used to create the contrast map. The contrast map decides what pixels should be sorted. Possible values: red, green, blue, hue, saturation, value.
7. Whether the program should print debug messages and create an image showing the contrast map. Setting this to "true" will show the progress of the calculations as well as create an image showing the contrast map at the cost of performance.

## How it works
### Brief explanation
The program first creates a "contrast map", this contrast map decides what pixels should be sorted and what pixels should be left alone based on the settings configuration. The program then clones the images and sorts "spans" from the contrast map and then saves the new image.
### In-depth explanation
#### Creating the contrast map
The contrast map decides whether a pixel will or will not be sorted. This is decided through the following logic: 
```
IF pixel.red IS GREATER THAN lower_bound AND pixel.red IS LESS THAN upper_bound THEN true ELSE false
``` 
or 
```
IF pixel.green IS GREATER THAN lower_bound AND pixel.green IS LESS THAN upper_bound THEN true ELSE false
```
etc etc.
#### Sorting the pixels
To sort the pixels, the program uses the contrast map to find what spans of pixels should be sorted. A span is defined as a length of pixels where the corresponding contrast map value is true. Given an example of a 4x4 image, where the number is the value being sorted and TRUE/FALSE denotes the contrast map value:<br />
```
{(1,TRUE) , (3,TRUE) , (2,TRUE) , (0,FALSE) }  
{(2,FALSE), (3,TRUE) , (1,TRUE) , (0, FALSE)}  
{(3,TRUE) , (2,FALSE), (0,FALSE), (1,TRUE)  }  
{(0,FALSE), (2,FALSE), (1,TRUE) , (3,FALSE) }
```
This would turn into when the sorting direction is set to right:
```
{(1,TRUE) , (2,TRUE) , (3,TRUE) , (0,FALSE) }  
{(2,FALSE), (1,TRUE) , (3,TRUE) , (0, FALSE)}  
{(3,TRUE) , (2,FALSE), (0,FALSE), (1,TRUE)  }  
{(0,FALSE), (2,FALSE), (1,TRUE) , (3,FALSE) }
```
And then the new sorted values then overwrite the old unsorted values.