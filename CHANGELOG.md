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
