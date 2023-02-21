# add-subs
add sub files in bulk to video files. The program uses **mkvmerge** to add merge the sub files and video files.


## Assumptions
* Assumes that video files are of the same format.
* Assumes that sub files are of the same format.
* Assumes that the video files and sub files are the only files in the directory of the specified formats.
* Assumes that the file names of the video and sub files are named that they would be in order when put in alphabetical order.
* Assumes you will be adding sub files of a languge that is: English, Spanish, Japanese, or Undetermined.


## Requirements
* mkvtoolnix

## Error Checkers
* verify that the language choice complies with [ISO 639.2](https://www.loc.gov/standards/iso639-2/php/code_list.php) as required by mkvmerge.
* verify that there is one sub file for each video file.
* asks user to confirm which sub file is going to which video file.
