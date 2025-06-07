# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.0.1] - 2022-04-15

### Added

- Analysis: analyze the resource images in the MD file, list the network and local , and alarm if the local does not exist
- Clean up: Check all the images in the directory where the MD file's local images are located . If they are not linked in the MD file, they will be moved to the "md-img-remove" directory
- Download: Download all network images in the MD file to the defined directory
- Upload: Automatically upload all local images in the MD file to the PicBed through picgo tool
- Move: move all local images in the MD file to the defined directory

## [0.0.2] - 2022-04-16

### Added

- Clean up the invalid local image in the MD file

### Removed

- remove the  PicGo package and will link global PicGo when use

## [0.0.3] - 2022-04-17

### Added

- Fix the bug when upload to aliyun by Picgo
- Add the switch , whether still update picture link when selection/position changed

## [0.0.4] - 2022-04-17

### Added

- Fix the bug when create MD file

## [0.0.5] - 2022-04-19

### Added

- Update the image in readme.md
- Add the 'Marndown MIM' in command for search more easily

## [0.0.6] - 2022-04-20

### Added

- Add timeout of download and upload
- Fix the invalid filename when download some URL

## [0.0.7] - 2022-04-20

### Added

- The limit of VSCode change to 1.46.0

## [0.0.8] - 2022-05-03

### Added

1. support paste clipboard image and define image's path
2. fix the bug, can download images when HTTPS is unauthorized
3. optimize the analysis of images mark
4. support the  escaped image URL
5. convert the format of selected image URL escape or not

## [0.0.9] - 2022-10-18

### Added

1. convert local image URL relative to absolute / absolute to relative

## [0.1.0] - 2022-10-19

Use esbuild

## [0.1.1] - 2023-6-25

Some bugs were fixed based on the code contributed by shaolonger.

1. When creating a local folder, a recursive method is used to ensure success even when intermediate levels do not exist.
2. Fixed the bug where an abnormal error was reported when the VSCode language was not within the default values.
3. Added a new function that, when the image extension does not exist  after removing illegal characters, automatically obtains and fills back  the extension based on the file buffer.
4. Fixed the problem where the program exited abnormally due to illegal characters in the URL address when encoding and decoding the URL.

## [0.1.2] - 2025-2-23

1. Local files support images in the `<img src="">` format, adopting the PR from Romantoscalion.
2. Custom renaming formats are supported when renaming files. For details, see Readme.md -> Renaming Rules.
3. Added a menu option: Copy Local Images. This option copies the images to a new directory and updates the link content.
4. The limit of VSCode change to 1.75

## [0.1.3] - 2025-6-7

1. Optimize and support the image replacement mode in the `<img src="">` format.
2. Support `![  alt text ]`  `(imagepath "Optional title")`  and  ` ![ ] ` `(<imagepath>)`.
3. Spaces are allowed in the path. If there is a `)` symbol in the path, it needs to be replaced with `%29`.

