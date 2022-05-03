# markdown-image-manage

> The VSCode extension of  [markdown-image-manage@github](https://github.com/pattazl/markdown-image-manage/).

[![installs](https://img.shields.io/vscode-marketplace/d/AustinYoung.markdown-image-manage.svg?style=flat-square)](https://marketplace.visualstudio.com/items?itemName=AustinYoung.markdown-image-manage)
[![GitHub stars](https://img.shields.io/github/stars/pattazl/markdown-image-manage.svg?style=flat-square&label=github%20stars)](https://github.com/pattazl/markdown-image-manage)
[![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg?style=flat-square)](https://standardjs.com)

## Overview

 "markdown-image-manage"  is the VSCdoe extension, manage the images of MD(Markdown) File by menu, and have another npm cli tool( markdown-image-manage-npm )

 [中文文档 Gitee ](https://gitee.com/pattazl/markdown-image-manage)  [中文文档 Github](https://github.com/pattazl/markdown-image-manage/blob/main/README_ZH.md)

## Features

The Menu show :

![cn menu](https://s2.loli.net/2022/04/19/EK1YlQPZeM7zDNc.png)

![en menu](https://s2.loli.net/2022/04/19/XpqtGoASJw6iCW4.png)

Functions as following :


1. Analysis: analyze the resource images in the MD file, list the network and local , and alarm if the local does not exist
2. Clean up: Check all the images in the directory where the MD file's local images are located , make sure the local image's folder is the same, if not same you can use **move local image** to make them same. If they are not linked in the MD file, they will be moved to the "md-img-remove" directory, the local images include .png,.jpg,.bmp,.gif,.jpeg,.ico,.tga,.rle,.tif,.cur,.ani,.iff 
3. Clean up the invalid local image:  Remove the link that the local image not exist.
4. Download: Download all network images in the MD file to the defined directory
5. Upload: Automatically upload all local images in the MD file to the PicBed (Depend on picgo )
6. Upload Clipboard: Upload the clipboard's image and insert the image  (Depend on picgo )
7. Move: move all local images in the MD file to the defined directory
8. The local and remote path support the variable ,  `<filename>` , `<YYYYMMDD>`  datetime format string by dayjs ，such as `<DDHHmm>` and so on.
9. Support paste clipboard image and define image's path
10. Support the escaped/unescaped image URL
11. Convert the format of selected image URL escape or unescape
12. Support multiple languages

## Requirements

Need global install PicGo tool when upload image

For make the install package smaller, markdown-image-manage haven't package the Picgo inside ，and link the global PicGo 

Step of install PicGo：

1.  Install node.js 12 and above
2.  Cli  `npm install picgo -g`，使用方法见  https://picgo.github.io/PicGo-Core-Doc/ 相关的仓库为 https://github.com/PicGo/PicGo-Core 

Picgo's introduction in  https://picgo.github.io/PicGo-Core-Doc/  and  https://github.com/PicGo/PicGo-Core 

After install Picgo , you should set the PicBed's account, and could use cli `picgo u /x/xxx.png` for test upload images

Picgo support several PicBed and  FTP server( PASV mode) by plugin.

##  Extension Settings

Get details in the extension, some of setting as below:

1. removeFolder: The folder where image will move to when clean local image folder
2. hasBracket: Whether the image path include right bracket
3. imageSaveFolder: Local folder which the images will save to, support absolute or relative path. support `<filename>` and date format`<YYYYMMDD>` variable (dayjs)
4. updateLink: Whether update the picture link in md file(Clean,Download,Upload,Move)
5. skipSelectChange: Whether still update picture link when selection/position changed
6. rename: Whether rename the image files(Download,Upload,Move
7. remotePath: Which be added at beginning of PicBed path, support `<filename>` and date format`<YYYYMMDD>` variable (dayjs)
8. clipboardPath: Clipboard image's path and name, support <filename> and date format<YYYYMMDD> variable (dayjs)
9. urlFormatted: Whether escape image URL when insert local image

##  Known Issues

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

