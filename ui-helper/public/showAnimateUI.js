let monitorInfo = null; // [{ "Left": -2880, "Top": 0, "Right": -1440, "Bottom": 900 }, { "Left": 0, "Top": 0, "Right": 2560, "Bottom": 1440 }]
let newMonitor = [] // 转换后的监视器信息
let globalScale = 1;
let globalOffsetX = 0;
let globalOffsetY = 0;
//let globalinFullScreen = false
//let globalSmallScale = 1; // 恢复常态时的比例
let globalMonitor = 1
let timeOutList = [] // 产生新text的定时器句柄
let objContainer = null
let objMain = null
// 创建一个窗口显示 Div
let winOpt = {
    guiWidth: 240,
    guiHeigth: 0,
    guiBgcolor: "8611AA",
    guiBgTrans: 0,
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
    guiMonitorNum: 1, // 第几个显示器
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
    ctrlHeight: 0,
    ctrlBgcolor: "11AA99",
    ctrlOpacity: 150,
    ctrlTextFont: "Verdana",
    ctrlTextSize: 20,
    ctrlTextWeight: "bold",
    ctrlTextColor: "FF0000",

    guiRadius: 0,
    ctrlRadius: 0,
    guiFadeMs: 500,
}


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
    let direction = ''
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
// 清空相关对象
function clearAll() {
    // 创建前先清空DIV
    clearDivs(objContainer)
    clearTimeout() // 清理定时器
}
function createAnimatedDivs() {
    clearAll()
    /*
    console.log('createAnimatedDivs')
    if(globalinFullScreen){
        debugger
    }  */
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
    newDiv.classList.add('demo-ctrl-div');
    let scale = globalScale
    // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
    newDiv.textContent = '';
    newDiv.style.width = winOpt.ctrlWidth * scale + 'px';
    if (winOpt.ctrlHeight > 0) {
        newDiv.style.height = winOpt.ctrlHeight * scale + 'px';
    }
    newDiv.style.borderRadius = winOpt.ctrlRadius * scale + 'px';
    newDiv.style.fontSize = winOpt.ctrlTextSize * scale + 'px';
    newDiv.style.fontFamily = winOpt.ctrlTextFont
    let bgColor = ''
    if (winOpt.guiBgTrans == 1) {
        bgColor = 'transparent'
    } else {
        bgColor = '#' + winOpt.ctrlBgcolor
    }
    newDiv.style.backgroundColor = bgColor
    newDiv.style.opacity = winOpt.ctrlOpacity / 255
    newDiv.style.lineHeight = winOpt.ctrlTextSize * scale + 'px';
    newDiv.style.fontWeight = winOpt.ctrlTextWeight
    newDiv.style.color = '#' + winOpt.ctrlTextColor

    newDiv.style.top = (globalOffsetY + winOpt.ctrlY) * scale + 'px'
    newDiv.style.left = (globalOffsetX + winOpt.ctrlX) * scale + 'px'

    mainAddInfo(newDiv, '.demo-ctrl-div')
    // 3s后清理掉
    textArr.forEach((text, i) => {
        // 间隔多少秒产生新的
        let handle = setTimeout(() => {
            newDiv.textContent = text
        }, 1000 * i);
        timeOutList.push(handle)
    })
}
// 对某个对象渐变消失
function fadeDiv(targetElement) {
    // 动态设置渐变时长（单位：毫秒）
    const fadeDuration = winOpt.guiFadeMs;
    // 获取元素初始透明度
    const initialOpacity = parseFloat(getComputedStyle(targetElement).opacity);
    // 记录开始时间
    const startTime = performance.now();
    function fade() {
        const currentTime = performance.now();
        const elapsedTime = currentTime - startTime;
        // 根据已过去的时间计算当前透明度
        const currentOpacity = initialOpacity - (elapsedTime / fadeDuration) * initialOpacity;

        if (currentOpacity > 0) {
            targetElement.style.opacity = currentOpacity;
            requestAnimationFrame(fade);
        } else {
            targetElement.style.opacity = 0;
            // 渐变完成后移除元素
            if (targetElement.parentNode != null) {
                targetElement.parentNode.removeChild(targetElement);
            }
        }
    }
    requestAnimationFrame(fade);
}
function showApp() {
    let textArr = ['SysDefault', 'Desktop', 'C:\\Windows\\explorer.exe', 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe']
    const newDiv = document.createElement('div');
    newDiv.classList.add('demo-app-div');
    let scale = globalScale
    let fontSize = 12
    // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
    newDiv.textContent = '';
    newDiv.style.fontSize = fontSize * scale + 'px'; // 默认12号字体
    newDiv.style.backgroundColor = '#F5F5F5'
    newDiv.style.lineHeight = fontSize * scale + 'px';

    newDiv.style.top = (globalOffsetY + winOpt.activeAppShowY) * scale + 'px'
    newDiv.style.left = (globalOffsetX + winOpt.activeAppShowX) * scale + 'px'
    mainAddInfo(newDiv, '.demo-app-div')
    // 3s后清理掉
    textArr.forEach((text, i) => {
        // 间隔多少秒产生新的
        let handle = setTimeout(() => {
            newDiv.textContent = text
        }, 1000 * i);
        timeOutList.push(handle)
    })
}
function mainAddInfo(newDiv, className) {
    // 需要先移除旧的
    try {
        let olds = document.querySelectorAll(className)
        olds.forEach(x => {
            objMain.removeChild(x);
        })
        objMain.appendChild(newDiv);
    } catch (e) { }
}
function showKey() {
    let textArr = ['k e y', 'p r e s×2', 't e s t', '🖱️×3 ^+v', '⇧+c ␣×12', '⊞+d', 'Caps', 'Del×4'];
    textArr = textArr.map(x => {
        return x.replace(/ /g, winOpt.txtSplit);
    })
    textArr.forEach((text, i) => {
        // 间隔多少秒产生新的
        let handle = setTimeout(() => {
            createNewTxt(text)
        }, winOpt.guiInterval * i);
        timeOutList.push(handle)
    })
}
let lastNewDiv = null
function createNewTxt(text) {
    let scale = globalScale
    const newDiv = document.createElement('div');
    lastNewDiv = newDiv;
    newDiv.classList.add('demo-new-div');
    // 需要根据参数创建 guiWidth: 240, guiHeigth: 0, guiBgcolor: "8611AA", guiBgTrans: 0, guiTrans: 1, guiOpacity: 38, guiTextFont: "Verdana", guiTextSize: 26, guiTextWeight: "bold", guiTextColor: "FFEE00",
    newDiv.textContent = text;
    if (winOpt.guiEdge == 1) {
        newDiv.style.border = '1px solid'
    }
    newDiv.style.textAlign = "left"
    newDiv.style.width = winOpt.guiWidth * scale + 'px';
    newDiv.style.borderRadius = winOpt.guiRadius * scale + 'px';
    newDiv.style.fontSize = winOpt.guiTextSize * scale + 'px';
    newDiv.style.fontFamily = winOpt.guiTextFont
    newDiv.style.color = '#' + winOpt.guiTextColor
    let editHeight = winOpt.guiTextSize
    if (winOpt.guiHeigth != 0) {
        newDiv.style.height = winOpt.guiHeigth * scale + 'px';
        editHeight = winOpt.guiHeigth
    }else{
        let padPx = 2
        newDiv.style.padding = padPx * scale + 'px' // 有间隔
        editHeight = editHeight + 2*padPx;
    }
    editHeight *= scale
    let bgColor = ''
    if (winOpt.guiBgTrans == 1) {
        bgColor = 'transparent'
    } else {
        bgColor = '#' + winOpt.guiBgcolor
    }
    newDiv.style.backgroundColor = bgColor
    newDiv.style.opacity = winOpt.guiOpacity / 255

    newDiv.style.lineHeight = winOpt.guiTextSize * scale + 'px';
    newDiv.style.fontWeight = winOpt.guiTextWeight
    newDiv.style.color = '#' + winOpt.guiTextColor

    let { guiX, guiY } = getInitPos(editHeight)
    newDiv.style.top = guiY * scale + 'px'
    newDiv.style.left = guiX * scale + 'px'
    objContainer.appendChild(newDiv);
    const existingDivs = objContainer.querySelectorAll('.demo-new-div');
    existingDivs.forEach(div => {
        if (div !== newDiv) {
            let direction = getDirection()
            let prePos = 0, selfSize = 0, offset = winOpt.guiMargin;
            switch (direction) {
                case 'up':
                    prePos = parseInt(getComputedStyle(div).top) || 0;
                    selfSize = div.offsetHeight //parseInt(getComputedStyle(div).height)
                    div.style.top = (prePos - offset - selfSize) + 'px';
                    break;
                case 'down':
                    prePos = parseInt(getComputedStyle(div).top) || 0;
                    selfSize = div.offsetHeight //parseInt(getComputedStyle(div).height)
                    div.style.top = (prePos + offset + selfSize) + 'px';
                    break;
                case 'left':
                    prePos = parseInt(getComputedStyle(div).left) || 0;
                    selfSize = div.offsetWidth //parseInt(getComputedStyle(div).width)
                    div.style.left = (prePos - offset - selfSize) + 'px';
                    break;
                case 'right':
                    prePos = parseInt(getComputedStyle(div).left) || 0;
                    selfSize = div.offsetWidth //parseInt(getComputedStyle(div).width)
                    div.style.left = (prePos + offset + selfSize) + 'px';
                    break;
            }
        }
    });
    // 超时后清理掉
    setTimeout(() => {
        try {
            // objContainer.removeChild(newDiv)
            fadeDiv(newDiv)
        } catch (e) { }
    }, winOpt.guiLife);
}
// 要取最大范围，转换 monitorInfo 的负数位置
function getOffset(main) {
    let demoWidth = parseInt(getComputedStyle(main).width)
    // console.log('demoWidth', demoWidth)
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
    // console.log('scale', maxInfo.Bottom - maxInfo.Top)
    let height = parseInt(scale * (maxInfo.Bottom - maxInfo.Top))

    // 调整高度
    main.style.height = height + 'px'
    // globalSmallScale = scale  // 小的比例
    globalScale = scale      // 引用的比例
    // 设置全局比例偏差
    globalOffsetX = offsetX
    globalOffsetY = offsetY
    //return { offsetX, offsetY, scale }
}
// 判断内容是否被清空,寻找动画层是否存在
function findDivs() {
    parent = document.getElementById('monitorId' + globalMonitor)
    if (parent == null) return true
    const allDivs = parent.querySelectorAll('div');
    return allDivs.length > 0;
}
function clearDivs(parent) {
    if (parent == null) return
    const childDivs = parent.querySelectorAll('div');
    childDivs.forEach(div => {
        div.remove();
    });
}
// getNewInfo()
let langText = {}
function setLangText(obj) {
    langText = obj
    showSize() // 刷新显示下
}
function showKeyPress(opt){
    let port = opt?.common?.serverPort??0
    if(port ==0)return;
    try {
        const ws = new WebSocket('ws://127.0.0.1:'+port);
        ws.onopen = () => {
        console.log('已连接到服务器');
        // 发送字符串（浏览器会自动转换为二进制）
        ws.send('ahkKeyShow'); // 表明要收的数据
        };
        ws.onmessage = async (event) => {
        const blob = event.data;
        // 直接将 Blob 转为 UTF-8 字符串（默认编码，无需额外配置）
        let text = blob instanceof Blob ? await blob.text() : blob;
        if(text.indexOf('0::')>-1){
            //showNewTxt(text)
            text = text.replace('0::','')
            lastNewDiv.textContent = text
        }else{
            createNewTxt(text)
        }
        
        // console.log('解码后的字符串:', text);
        };
    } catch (e) {
        alert('ws:'+e.message)
    }
}
function initContain(monitor, opt) {
    if (monitor == null) {
        return;
    }
    //fullScreenTxt = fsTxt
    monitorInfo = monitor
    objMain = document.getElementById("mainContain")
    if (objMain == null) return;
    // observer.observe(objMain); // 暂时不必自动根据大小更新
    // 更新参数并刷新
    updateWinOpt(opt)
    // 如果是配置界面则动态演示
    if(location.pathname.indexOf('Setting')>-1){
        keepAnimate()
    }else{
        showKeyPress(opt)
    }
    // fullScreenEvent
    // fullScreenEvent()
}

// 显示当前的窗口像素 根据子窗口的大小显示真实长宽
function showSize() {
    setTimeout(() => {
        if (objMain == null) return;
        let monitors = objMain.querySelectorAll('.demo-color-changing-div')
        monitors.forEach((dom, i) => {
            let w = parseInt(getComputedStyle(dom).width);
            let h = parseInt(getComputedStyle(dom).height);
            dom.querySelector('span').innerText = `${langText.intro187 ?? ''} #${i + 1}(${w}x${h}) scale:${(globalScale*100).toFixed(2)}%`
            // (dom as any).title = `${contentText.value.intro187}(${w}*${h})`
            //console.log(dom, w, h)
        })
    }, 10)
}
function changeOK() {
    let obj = document.getElementById("colorValue")
    if (obj == null) return;
    let monitors = objMain.querySelectorAll('.demo-color-changing-div')
    monitors.forEach(x => {
        x.style.backgroundColor = obj.value;
    })
}
// 颜色列表
let colorArr = ['#FFFFFF', '#C8D7E3', '#7AC142', '#003366', '#0078D7', '#8E44AD', '#00B7EB']
function changeNext() {
    let obj = document.getElementById("colorValue")
    if (obj == null) return;
    let find = colorArr.findIndex(item => item === obj.value.toUpperCase());
    find = (++find) % colorArr.length;
    obj.value = colorArr[find]
    changeOK()
}
// 初始化主窗口,此函数需要防抖，防止频繁触发
let mainTimer = null
function initMain() {
    if (mainTimer) {
        clearTimeout(mainTimer);
    }
    let delay = 100
    mainTimer = setTimeout(() => {
        initMainCore();
        mainTimer = null; // 清空定时器标识
    }, delay); //  防抖延迟
}
function initMainCore() {
    clearAll()
    getOffset(objMain)
    let scale = globalScale, offsetX = globalOffsetX, offsetY = globalOffsetY
    // 计算偏移，修改为 Width 或 Height
    newMonitor = monitorInfo.map(x => {
        return { "Left": x.Left + offsetX, "Top": x.Top + offsetY, "Width": x.Right - x.Left, "Height": x.Bottom - x.Top }
    })
    // console.log('newMonitor:', JSON.stringify(newMonitor))
    let arrHTML = newMonitor.map((x, i) => {
        let screenHTML = `<div style="top:${x.Top * scale}px;left:${x.Left * scale}px;" class="demo-container">
        <div id="monitorId${i + 1}" oriWidth="${x.Width}" oriHeight="${x.Height}" style="width:${x.Width * scale}px;height:${x.Height * scale}px;" class="demo-color-changing-div">
        <span style="background-color:lightgrey;color:black"></span>
        <span><input id="colorValue" placeholder="#FFFFFF" value="#FFFFFF" maxlength="7" style="width:${120 * scale}px"  oninput="this.value = this.value.replace(/[^#a-fA-F0-9]/g, '')" title="color" />
        <input type="button" name="btChangeOK" title="change background color" value="OK"/>
        <input type="button" value="➡️" title="next background color" name="btChangeNext"/>
        </span>
        </div>
    </div>`
        return screenHTML
    })
    objMain.innerHTML = arrHTML.join('')

    let objBtNext = document.getElementsByName("btChangeNext")
    let objBtOK = document.getElementsByName("btChangeOK")
    objBtNext.forEach( o =>{ o.onclick = changeNext})
    objBtOK.forEach( o =>{ o.onclick = changeOK})

    // 设定具体哪个模块中显示
    let monitorIndex = winOpt.guiMonitorNum
    if (monitorIndex > monitorInfo.length || monitorIndex == 0) {
        monitorIndex = 1
    }
    globalMonitor = monitorIndex
    objContainer = document.getElementById('monitorId' + globalMonitor); // 容器显示
    showSize()
}
// 保持动画一直播放
function keepAnimate() {
    setInterval(() => { if (!findDivs()) { createAnimatedDivs(); } }, 1000)
}
// 参数变化需要更新,更新后需要强刷
function updateWinOpt(opt) {
    Object.entries(winOpt).forEach(([key]) => {
        if (opt.dialog[key] !== undefined) {
            winOpt[key] = opt.dialog[key];
            return
        }
        if (opt.common[key] !== undefined) {
            winOpt[key] = opt.common[key];
        }
    });
    // 后续修改值，需要刷新内容
    initMain()
}
function changeContainSize(flag) {
    let main = document.getElementById('mainContain');
    let defaultWidth = '100%'  // 同CSS中一致
    let preWidth = main.style.width || defaultWidth
    if (flag == 0) {
        main.style.width = defaultWidth
    } else if (flag < 0) {
        main.style.width = (parseInt(preWidth) - 10) + "%"
    } else {
        main.style.width = (parseInt(preWidth) + 10) + "%"
    }
    initMain()
    showSize()
}
// 初始化容器
//initContain()
// 调用函数，传入容器 ID 和移动方向
//createAnimatedDivs();
//setInterval(() => { if (!findDivs(objContainer)) { createAnimatedDivs(); } }, 1000)
console.log('showAnimate', window)

export {
    setLangText,
    initContain,
    updateWinOpt,
    changeContainSize,
}


