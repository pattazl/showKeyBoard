    let monitorInfo = [{ "Left": -2880, "Top": 0, "Right": -1440, "Bottom": 900 }, { "Left": 0, "Top": 0, "Right": 2560, "Bottom": 1440 }]
    let newMonitor = [] // 转换后的监视器信息
    // 创建一个窗口显示 Div
    let winOpt = {
        guiWidth: 240,
        guiHeigth: 0,
        guiBgcolor: "8611AA",
        guiBgTrans: 0,
        guiTrans: 1,
        guiOpacity: 38,
        guiTextFont: "Verdana",
        guiTextSize: 26,
        guiTextWeight: "bold",
        guiTextColor: "FFEE00",
        guiLife: 7000,
        guiInterval: 1000,
        guiPos: "TL",
        guiPosXY: "Y",
        guiPosOffsetX: 50,
        guiPosOffsetY: 50,
        guiDpiscale: 0,
        guiMonitorNum: 2, // 第几个显示器
        guiMargin: 5,
        guiEdge: 1,
        txtSplit: " ",
        ctrlX: 500,
        ctrlY: 50,
        activeAppShowX: 600,
        activeAppShowY: 0,
        // 大开关
        needShowKey: 1, // 是否显示按键
        activeAppShow: 1, // 是否显示进程名
        ctrlState: 1,  // 是否显示控制键

        ctrlWidth: 240,
        ctrlBgcolor: "11AA99",
        ctrlOpacity: 150,
        ctrlTextFont: "Verdana",
        ctrlTextSize: 20,
        ctrlTextWeight: "bold",
        ctrlTextColor: "FF0000",
    }
    let globalScale = 1;
    let globalSmallScale = 1;
    let globalMonitor = 1
    let timeOutList = [] // 产生新text的定时器句柄
    let ctrlAppSec = 12000  // ctrl 和 app的显示时间

    function clearTimeout() {
        timeOutList.forEach(x => {
            clearInterval(x)
        })
        timeOutList = []
    }

    // 根据 宽度和位置确定初始位置
    function getInitPos(editHeight) {
        // 方向根据 guiPos 和 guiPosXY 决定 guiPosOffsetX: 0, guiPosOffsetY: -50,
        // 默认 TL
        let guiX = winOpt.guiPosOffsetX
        let guiY = winOpt.guiPosOffsetY

        let width = newMonitor[globalMonitor - 1].Width
        let height = newMonitor[globalMonitor - 1].Height
        // 此处坐标需要转换为不同屏幕的，每个屏幕默认起始点为 0 0
        switch (winOpt.guiPos) {
            case 'TR':
                guiX = guiX + width - winOpt.guiWidth
                break;
            case 'BL':
                guiY = guiY + height - editHeight
                break;
            case 'BR':
                guiX = guiX + width - winOpt.guiWidth
                guiY = guiY + height - editHeight
                break;
        }
        return { guiX, guiY }
    }
    // 根据参数获取移动方向
    function getDirection() {
        switch (winOpt.guiPosXY) {
            // 上下
            case 'Y':
                if (winOpt.guiPos.indexOf('T') > -1) {
                    // 向下
                    direction = 'down'
                } else {
                    // 向上
                    direction = 'up'
                }
                break;
            case 'X':
                if (winOpt.guiPos.indexOf('L') > -1) {
                    // 向右
                    direction = 'right'
                } else {
                    // 向左
                    direction = 'left'
                }
                break;
        }
        return direction
    }
    function createAnimatedDivs() {
        // 创建前先清空DIV
        clearDivs(objContainer)
        clearTimeout() // 清理定时器
        if (winOpt.needShowKey == 1) {
            showKey()
        }
        if (winOpt.ctrlState == 1) {
            showCtrl()
        }
        if (winOpt.activeAppShow == 1) {
            showApp()
        }
    }
    function showCtrl() {
        let textArr = ['Caps', '^ Caps', '⊞ ⎇', '^ ⇧ ⎇', '^ ⇧ ⊞ ⎇', '^ ⇧ ⊞ ⎇ Caps'];
        const newDiv = document.createElement('div');
        newDiv.classList.add('ctrl-div');
        let scale = globalScale
        // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
        newDiv.textContent = '';
        newDiv.style.width = winOpt.ctrlWidth * scale + 'px';
        newDiv.style.fontSize = winOpt.ctrlTextSize * scale + 'px';
        newDiv.style.fontFamily = winOpt.ctrlTextFont
        newDiv.style.backgroundColor = '#' + winOpt.ctrlBgcolor
        newDiv.style.opacity = winOpt.ctrlOpacity / 255
        newDiv.style.lineHeight = winOpt.ctrlTextSize * scale + 'px';
        newDiv.style.fontWeight = winOpt.ctrlTextWeight
        newDiv.style.color = '#' + winOpt.ctrlTextColor

        newDiv.style.top = winOpt.ctrlY * scale + 'px'
        newDiv.style.left = winOpt.ctrlX * scale + 'px'
        objContainer.appendChild(newDiv);
        // 3s后清理掉
        textArr.forEach((text, i) => {
            // 间隔多少秒产生新的
            let handle = setTimeout(() => {
                newDiv.textContent = text
            }, 1000 * i);
            timeOutList.push(handle)
        })
        let h = setTimeout(() => {
            try { objContainer.removeChild(newDiv); } catch (e) { }
        }, ctrlAppSec);
        timeOutList.push(h)
    }
    function showApp() {
        let textArr = ['SysDefault', 'Desktop','C:\\Windows\\explorer.exe', 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe']
        const newDiv = document.createElement('div');
        newDiv.classList.add('app-div');
        let scale = globalScale
        let fontSize = 12
        // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
        newDiv.textContent = '';
        newDiv.style.fontSize = fontSize * scale + 'px'; // 默认12号字体
        newDiv.style.backgroundColor = '#F5F5F5'
        newDiv.style.lineHeight = fontSize * scale + 'px';

        newDiv.style.top = winOpt.activeAppShowY * scale + 'px'
        newDiv.style.left = winOpt.activeAppShowX * scale + 'px'
        objContainer.appendChild(newDiv);
        // 3s后清理掉
        textArr.forEach((text, i) => {
            // 间隔多少秒产生新的
            let handle = setTimeout(() => {
                newDiv.textContent = text
            }, 1000 * i);
            timeOutList.push(handle)
        })
        let h = setTimeout(() => {
            try { objContainer.removeChild(newDiv); } catch (e) { }
        }, ctrlAppSec);
        timeOutList.push(h)
    }
    function showKey() {
        let textArr = ['k e y', 'p r e s×2', 't e s t', '🖱️×3 ^+v', '⇧+c ␣×12', '⊞+d', 'Caps', 'Del×4'];
        textArr = textArr.map(x => {
            return x.replace(/ /g, winOpt.txtSplit);
        })
        textArr.forEach((text, i) => {
            // 间隔多少秒产生新的
            let handle = setTimeout(() => {
                createNewTxt(text, globalScale)
            }, winOpt.guiInterval * i);
            timeOutList.push(handle)
        })
    }
    function createNewTxt(text, scale) {
        const newDiv = document.createElement('div');
        newDiv.classList.add('new-div');
        // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
        newDiv.textContent = text;
        if(winOpt.guiEdge == 1){
            newDiv.style.border = '1px solid'
        }
        newDiv.style.width = winOpt.guiWidth * scale + 'px';
        newDiv.style.fontSize = winOpt.guiTextSize * scale + 'px';
        newDiv.style.fontFamily = winOpt.guiTextFont
        let editHeight = winOpt.guiTextSize
        if (winOpt.guiHeigth != 0) {
            newDiv.style.height = winOpt.guiHeigth * scale + 'px';
            editHeight = winOpt.guiHeigth
        }
        editHeight *= scale
        console.log(editHeight)
        newDiv.style.backgroundColor = '#' + winOpt.guiBgcolor
        newDiv.style.opacity = winOpt.guiOpacity / 255

        newDiv.style.lineHeight = winOpt.guiTextSize * scale + 'px';
        newDiv.style.fontWeight = winOpt.guiTextWeight
        newDiv.style.color = '#' + winOpt.guiTextColor

        let { guiX, guiY } = getInitPos(editHeight)
        newDiv.style.top = guiY * scale + 'px'
        newDiv.style.left = guiX * scale + 'px'
        objContainer.appendChild(newDiv);
        const existingDivs = objContainer.querySelectorAll('.new-div');
        existingDivs.forEach(div => {
            if (div !== newDiv) {
                let direction = getDirection()
                let prePos = 0, selfSize = 0, offset = winOpt.guiMargin;
                switch (direction) {
                    case 'up':
                        prePos = parseInt(getComputedStyle(div).top) || 0;
                        selfSize = parseInt(getComputedStyle(div).height)
                        div.style.top = (prePos - offset - selfSize) + 'px';
                        break;
                    case 'down':
                        prePos = parseInt(getComputedStyle(div).top) || 0;
                        selfSize = parseInt(getComputedStyle(div).height)
                        div.style.top = (prePos + offset + selfSize) + 'px';
                        break;
                    case 'left':
                        prePos = parseInt(getComputedStyle(div).left) || 0;
                        selfSize = parseInt(getComputedStyle(div).width)
                        div.style.left = (prePos - offset - selfSize) + 'px';
                        break;
                    case 'right':
                        prePos = parseInt(getComputedStyle(div).left) || 0;
                        selfSize = parseInt(getComputedStyle(div).width)
                        div.style.left = (prePos + offset + selfSize) + 'px';
                        break;
                }
            }
        });
        // 超时后清理掉
        setTimeout(() => {
            try { objContainer.removeChild(newDiv); } catch (e) { }
        }, winOpt.guiLife);
    }
    // 要取最大范围，转换 monitorInfo 的负数位置
    function getOffset(demoWidth) {
        let maxInfo = { "Left": 0, "Top": 0, "Right": 0, "Bottom": 0 }  // 最小的 Left Top ，最大的 Right Bottom
        monitorInfo.forEach(x => {
            maxInfo.Left = Math.min(x.Left, maxInfo.Left)
            maxInfo.Top = Math.min(x.Top, maxInfo.Top)
            maxInfo.Right = Math.max(x.Right, maxInfo.Right)
            maxInfo.Bottom = Math.max(x.Bottom, maxInfo.Bottom)
        })
        // 坐标全部变成正数
        let offsetX = 0, offsetY = 0
        if (maxInfo.Left < 0) {
            offsetX = - maxInfo.Left
        }
        if (maxInfo.Right < 0) {
            offsetY = - maxInfo.Right
        }
        let scale = demoWidth / (maxInfo.Right - maxInfo.Left)
        console.log('scale', maxInfo.Bottom - maxInfo.Top)
        let height = parseInt(scale * (maxInfo.Bottom - maxInfo.Top))
        // 需要根据最大最小计算缩放比，根据宽度计算缩放比，demoWidth
        return { offsetX, offsetY, scale, height }
    }
    function findDivs(parent) {
        const result = [];
        const allDivs = parent.querySelectorAll('div');
        return allDivs.length > 0;
    }
    function clearDivs(parent) {
        const childDivs = parent.querySelectorAll('div');
        childDivs.forEach(div => {
            div.remove();
        });
    }
    // getNewInfo()
    function initContain() {
        let main = document.getElementById("mainContain")
        // 获取宽度
        let { offsetX, offsetY, scale, height } = getOffset(parseInt(getComputedStyle(main).width))
        // 计算偏移，修改为 Width 或 Height
        newMonitor = monitorInfo.map(x => {
            return { "Left": x.Left + offsetX, "Top": x.Top + offsetY, "Width": x.Right - x.Left, "Height": x.Bottom - x.Top }
        })
        console.log('newMonitor:', JSON.stringify(newMonitor))

        main.style.height = height + 'px'
        let arrHTML = newMonitor.map((x, i) => {
            let screenHTML = `<div style="top:${x.Top * scale}px;left:${x.Left * scale}px;" class="container">
            <div id="monitorId${i + 1}" style="width:${x.Width * scale}px;height:${x.Height * scale}px;" class="color-changing-div">
                <button onclick="fullScreen(${i + 1})">全屏${i + 1}</button>
            </div>
        </div>`
            return screenHTML
        })
        main.innerHTML = arrHTML.join('')
        // 开始观察 myDiv 元素
        newMonitor.map((x, i) => {
            let myDiv = document.getElementById(`monitorId${i + 1}`)
            observer.observe(myDiv);
        })
        globalSmallScale = scale
        globalScale = scale
        let monitorIndex = winOpt.guiMonitorNum
        if (monitorIndex > newMonitor.length || monitorIndex == 0) {
            monitorIndex = 1
        }
        globalMonitor = monitorIndex
    }
    // 全屏代码
    function fullScreen(id) {
        const myDiv = document.getElementById('monitorId' + id);
        toggleFullscreen(myDiv)
    }

    function toggleFullscreen(myDiv) {
        if (document.fullscreenElement) {
            if (document.exitFullscreen) {
                document.exitFullscreen();
            } else if (document.webkitExitFullscreen) {
                document.webkitExitFullscreen();
            } else if (document.mozCancelFullScreen) {
                document.mozCancelFullScreen();
            } else if (document.msExitFullscreen) {
                document.msExitFullscreen();
            }
        } else {
            if (myDiv.requestFullscreen) {
                myDiv.requestFullscreen();
            } else if (myDiv.webkitRequestFullscreen) {
                myDiv.webkitRequestFullscreen();
            } else if (myDiv.mozRequestFullScreen) {
                myDiv.mozRequestFullScreen();
            } else if (myDiv.msRequestFullscreen) {
                myDiv.msRequestFullscreen();
            }

        }
    }
    document.addEventListener('fullscreenchange', function () {
        console.log('fullscreenchange', document.fullscreenElement)
        if (document.fullscreenElement == null) {
            globalScale = globalSmallScale // 退回普通状态
            createAnimatedDivs()
        } else {
            if (document.fullscreenElement == objContainer) {
                globalScale = 1
                createAnimatedDivs()
            }
        }
    });

    document.addEventListener('fullscreenerror', function () {
        console.error('全屏操作出错');
    });
    // 监控变化
    const observer = new ResizeObserver(entries => {
        // for (const entry of entries) {
        //     const { width, height } = entry.contentRect;
        //     // 需要重新演示动画
        //     console.log(entry.target, objContainer)
        //     if (entry.target == objContainer) {
        //         createAnimatedDivs()
        //         console.log('createAnimatedDivs2222')
        //     }
        // }
    });
    // 初始化容器
    initContain()
    let objContainer = document.getElementById('monitorId' + globalMonitor); // 容器显示
    // 调用函数，传入容器 ID 和移动方向
    //createAnimatedDivs();
    setInterval(() => { if (!findDivs(objContainer)) { createAnimatedDivs(); } }, 1000)