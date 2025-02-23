# markdown-image-manage

> The VSCode extension of  [markdown-image-manage@github](https://github.com/pattazl/markdown-image-manage/).

[![installs](https://img.shields.io/vscode-marketplace/d/AustinYoung.markdown-image-manage.svg?style=flat-square)](https://marketplace.visualstudio.com/items?itemName=AustinYoung.markdown-image-manage)
[![GitHub stars](https://img.shields.io/github/stars/pattazl/markdown-image-manage.svg?style=flat-square&label=github%20stars)](https://github.com/pattazl/markdown-image-manage)
[![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg?style=flat-square)](https://standardjs.com)

## Overview

 "markdown-image-manage"  is the VSCode extension, manage the images of MD(Markdown) File by menu, and have another npm cli tool( markdown-image-manage-npm )

 [中文文档 Gitee ](https://gitee.com/pattazl/markdown-image-manage)  [中文文档 Github](https://github.com/pattazl/markdown-image-manage/blob/main/README_ZH.md)

## Features

The Menu show :
![en.png](https://s2.loli.net/2022/10/19/jqBMm62zShfPHex.png)

![cn menu](https://s2.loli.net/2022/10/19/YoX2rpUWaHgezPi.png)

Functions as following :

1. Analysis: analyze the resource images in the MD file, list the network and local , and alarm if the local does not exist
2. Clean up: Check all the images in the directory where the MD file's local images are located , make sure the local image's folder is the same, if not same you can use **move local image**  or **copy local image** to make them same. If they are not linked in the MD file, they will be moved to the "md-img-remove" directory, the local images include .png,.jpg,.bmp,.gif,.jpeg,.ico,.tga,.rle,.tif,.cur,.ani,.iff
3. Move/Copy: move or copy all local images in the MD file to the defined directory
4. Clean up the invalid local image:  Remove the link that the local image not exist.
5. Download: Download all network images in the MD file to the defined directory
6. Upload: Automatically upload all local images in the MD file to the PicBed (Depend on picgo )
7. Upload Clipboard: Upload the clipboard's image and insert the image  (Depend on picgo )
8. The local and remote path support the variable ,  `<filename>` , `<YYYYMMDD>`  datetime format string by dayjs ，such as `<DDHHmm>` and so on.
9. Support paste clipboard image and define image's path
10. Support the escaped/unescaped image URL
11. Convert the format of selected image URL escape or unescape
12. Convert the path of selected local image URL relative to absolute/absolute to relative
13. Support multiple languages
14. Support rename the new file name according to the rules.

## Requirements

Need global install PicGo tool when upload image

For make the install package smaller, markdown-image-manage haven't package the Picgo inside ，and link the global PicGo

Step of install PicGo：

1. Install node.js 12 and above
2. Cli  `npm install picgo -g`，使用方法见  https://picgo.github.io/PicGo-Core-Doc/ 相关的仓库为 https://github.com/PicGo/PicGo-Core

Picgo's introduction in  https://picgo.github.io/PicGo-Core-Doc/  and  https://github.com/PicGo/PicGo-Core

After install Picgo , you should set the PicBed's account, and could use cli `picgo u /x/xxx.png` for test upload images

Picgo support several PicBed and  FTP server( PASV mode) by plugin.

## Extension Settings

Get details in the extension, some of setting as below:

1. removeFolder: The folder where image will move to when clean local image folder
2. hasBracket: Whether the image path include right bracket
3. matchAngleBrackets: Whether match the image link in angle brackets
4. imageSaveFolder: Local folder which the images will save to, support absolute or relative path. support `<filename>` and date format `<YYYYMMDD>` variable (dayjs)
5. updateLink: Whether update the picture link in md file(Clean,Download,Upload,Move)
6. skipSelectChange: Whether still update picture link when selection/position changed
7. rename: Whether rename the image files(Download,Upload,Move,Copy)
8. nameFormat: the format of rename rules, default is [D] ,  For details, see  [Renaming Rules](### Renaming Rules)
9. remotePath: Which be added at beginning of PicBed path, support `<filename>` and date format `<YYYYMMDD>` variable (dayjs)
10. clipboardPath: Clipboard image's path and name, support `<filename>` and date format `<YYYYMMDD>` variable (dayjs)
11. urlFormatted: Whether escape image URL when insert local image


### Renaming Rules

The rules are similar to TotalCommand

1. [D] the default . The specific rule is a timestamp + a two - digit random number to ensure uniqueness.
2. [N] represents the original file name.
3. [N#-#] represents the character range of the original file name.
   1. [N1 - 2] means taking the first two letters of the file name.
   2. [N2 - 10] means starting from the second letter to the tenth letter of the file name.
   3. [N - 3] means taking the last three letters of the file name.
4. [C] represents automatic counting, starting from 1 by default.
5. [C0009] means the number has a length of 4 and starts counting from 9.
6. To prevent duplication, the rule must contain either [D] or [C]. If  either is missing, [C] will be automatically added at the end.
7. The above rules can be freely combined. The following are examples:
   1. For the rule `TEST-[C05]`, if the file name is "ABCDEF.jpg," etc., after modification, the file name will be TEST-05.jpg,TEST-06.jpg, ...
   2. For the rule `[N1-3]-[C002]`, if the file name is "ABCDEF.jpg,", etc.,  after modification, the file names will be ABC-002.jpg, ABC-003.jpg, ...

## Known Issues

None

## Release Notes

Refer to CHANGELOG.md

## Repository

https://github.com/pattazl/markdown-image-manage

https://gitee.com/pattazl/markdown-image-manage

the related npm cli tool : markdown-image-manage-npm

https://github.com/pattazl/markdown-image-manage-npm

https://gitee.com/pattazl/markdown-image-manage-npm

**Enjoy!**
