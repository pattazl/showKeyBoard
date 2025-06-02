# markdown-image-manage

> VSCode 插件  [markdown-image-manage@gitee](https://gitee.com/pattazl/markdown-image-manage/).

[![installs](https://img.shields.io/vscode-marketplace/d/AustinYoung.markdown-image-manage.svg?style=flat-square)](https://marketplace.visualstudio.com/items?itemName=AustinYoung.markdown-image-manage)
[![GitHub stars](https://img.shields.io/github/stars/pattazl/markdown-image-manage.svg?style=flat-square&label=github%20stars)](https://github.com/pattazl/markdown-image-manage)
[![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg?style=flat-square)](https://standardjs.com)

## 总览

Markdown图片管理工具 "markdown-image-manage" 在VSCode中用菜单管理markdown文件中的图片，另有相似功能的npm包 markdown-image-manage-npm 可供命令行中使用

## 功能

通过菜单操作

![cn menu](https://s2.loli.net/2022/10/19/YoX2rpUWaHgezPi.png)

![en.png](https://s2.loli.net/2022/10/19/jqBMm62zShfPHex.png)

可以实现如下功能

1. 将文中的所有的图片下载到指定目录
2. 清理文件中本地图片所在目录( 目录需一致，如果不一致可用**移动本地图片** 或 **复制本地图片**功能确保一致)，文章中如果未引用则转移，图片包括 .png,.jpg,.bmp,.gif,.jpeg,.ico,.tga,.rle,.tif,.cur,.ani,.iff 等
3. 移动/复制本地所有图片到指定目录并更新链接
4. 清理本地失效的图片链接
5. 分析文中的图片哪些是网络，哪些是本地，本地哪些失效
6. 将本地所有图片自动上传到图床的指定目录中（基于PicGo）
7. 上传剪切板中的图片（基于PicGo）
8. 本地下载路径和远端上传路径，均可设置文件名和日期变量， `<filename>` 表示文件名, `<YYYYMMDD>` 表示日期按(dayjs)模块格式化，比如 `<DDHHmm>`等
9. 支持从剪切板复制图片，并可自定义图片路径
10. 支持设置图片地址的是否转义
11. 可以对所选图片是否转义进行切换
12. 可以对所选本地图片是否使用相对路径进行切换
13. 支持多语言
14. 可以按规则重命名新的图片

## 依赖项

上传模块依赖PicGo

为了减少安装包大小，本程序没有内置picgo ，而是通过 npm link 方式关联全局的PicGo

需全局安装Picgo命令行版本，方法：

1. 首先需要确保已经安装了 node.js 12 以上版本
2. 命令行执行 `npm install picgo -g`，使用方法见  https://picgo.github.io/PicGo-Core-Doc/ 相关的仓库为 https://github.com/PicGo/PicGo-Core

安装后需要配置相关的图床帐号等信息，以便于顺利使用图床，建议用命令行 `picgo u /x/xxx.png` 测试是否可以成功上传。

picgo支持多种图床和各种插件，比如通过插件 picgo-plugin-ftp-uploader 实现上传到FTP服务器(被动模式)

## 插件配置

详情见插件中的参数说明，部分参数如下:

1. removeFolder: 本地图片目录清理时，其他图片移动的目标文件夹
2. hasBracket: 图片路径中是否包括右括号
3. matchAngleBrackets: 是否匹配尖括号中的图片链接
4. imageSaveFolder: 图片要保存到的位置,默认以md文件名做文件夹，支持绝对路径和相对路径，支持文件名 `<filename>`和日期 `<YYYYMMDD>`变量(dayjs)
5. updateLink: 上传/下载/移动时是否需要更新md文件的图片链接
6. skipSelectChange: 当光标或选择范围改变后是否依然更新图片链接
7. rename: 上传/下载/移动/复制图片时是否需要重命名图片
8. nameFormat: 重命名规则的字符串,默认为[D]
9. remotePath: 图床上需要添加的远程路径，可区分不同md文件的图片，支持文件名 `<filename>`和日期 `<YYYYMMDD>`变量(dayjs)。
10. clipboardPath: 剪切板图片的路径和名称，支持文件名 `<filename>`和日期 `<YYYYMMDD>`变量(dayjs)。
11. urlFormatted: 图片URL格式是否转义，转义后有兼容性强可读性弱，插入本地图片时生效

### 重命名规则

规则类似TotalCommand

1. [D]默认名,具体规则为时间戳+2位随机数,确保不重复
2. [N]表示原文件名
3. [N#-#]表示原文件名的字符范围
   1. [N1-2] 表示取文件名的开头2个字母
   2. [N2-10] 表示从文件名的第2个字母开始到第10个
   3. [N-3] 表示取文件名的最后3个字母
4. [C]表示自动计数默认从1开始
5. [C0009]表示数字长度为4,从9开始计数
6. 为防止重复,规则中必须包含[D]或[C]之一,如果缺少则自动在结尾加[C]
7. 以上规则可自由组合,以下为示例
   1. 规则 TEST-[C05] ,文件名为 ABCDEF.jpg  等 修改后,文件名为 TEST-05.jpg
   2. 规则 [N1-3]-[C002] ,文件名为 ABCDEF.jpg等 修改后,文件名为 ABC-002.jpg , ABC-003.jpg ...

## 已知问题

暂无

## 变更记录

### 0.0.1

基本功能，包括下载,移动,清理,分析,上传MD文件中的图片

### 0.0.2

#### 增加

增加清理本地失效图片功能

#### 移除

安装包中去掉PicGo，减小大小，通过执行上传时动态关联全局PicGo实现上传

### 0.0.3

#### 增加

增加开关，解决上传过程中光标或选择范围改变是否更新链接

解决Picgo上传阿里云的图床后图片无法更新的问题

### 0.0.4

#### 增加

修正创建MD文件时的提示和异常

### 0.0.5

#### 增加

- 更新readme中的图片来源为SMMS
- 添加 Marndown MIM 前缀到命令中更方便查找

### 0.0.6

#### 增加

- 添加下载和上传的超时
- 修复下载文件时候有异常文件名无法保存
- 修复本地文件重名时用[序号]标记

### 0.0.7

#### 增加

- VSCode限制改为 1.46.0

### 0.0.8

#### 增加

1. 支持粘帖剪切板图片到本地，路径和文件名可自定义
2. 修复bug，支持当https证书无效时依旧可下载图片
3. 优化图片识别算法，图片路径中支持智能识别括号
4. 支持插入图片图片时设置是否转义
5. 支持对所选图片进行是否转义的切换

## 0.0.9

#### 增加

1. 支持绝对路径和相对路径的切换（参考第一个路径）

## 0.1.0

1. 使用esbuild打包
2. 修复部分提示

## 0.1.1

根据 shaolonger 贡献的代码修复部分bug

1. 创建本地文件夹时采用递归方式，保证中间层级不存在时也能成功
2. 修复vscode语言未在默认值内时异常报错的bug
3. 新增剔除不合法字符后图片拓展名不存在的情况下，自动根据文件buffer获取并回填拓展名的功能
4. 修复将URL地址进行转义和还原时URL地址包含不合法字符异常退出的问题

## 0.1.2

1. 本地文件支持`<img src="">` 格式的图片分析，采用 Romantoscalion 的 PR

2. 文件重命名时候支持自定义重命名格式,具体见  [重命名规则](#重命名规则)

3. 增加菜单: 复制本地图片,将图片复制到新目录中并更新链接内容

4. VSCode限制改为 1.75

    

## 仓库地址

https://github.com/pattazl/markdown-image-manage

https://gitee.com/pattazl/markdown-image-manage

相关 markdown-image-manage-npm 仓库地址

https://github.com/pattazl/markdown-image-manage-npm

https://gitee.com/pattazl/markdown-image-manage-npm

**使用愉快!**
