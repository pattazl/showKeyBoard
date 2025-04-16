# Interface开发接口

## HTTP接口清单

默认均访问 127.0.0.1 的本地地址

### 检查服务

/version

返回 status == 200 即为成功

### 屏幕信息

/sendPCInfo

发送PC相关信息,比如屏幕信息

格式

```json
{
    "screen": 
    // 多个屏幕的信息
    [{
            "Left": 0, // 左上角
            "Top": 0,
            "Right": 2560, // 右下角
            "Bottom": 1440
        }
    ],
    "flag": 0  // flag=0 表示第一次发送数据 ,如果鼠标坐标数据异常会发送1,用于更新屏幕信息
}

```

### 退出复位

/exit

发送消息,后端web服务退出

### 操作信息

/data

发送具体数据

格式

```javascript
{	
  "按键名":次数,	 // 按键名支持组合按键类似<^c
  "App-Key-执行文件路径":次数, // App-Key-表示键盘按键次数
  "App-Mouse-执行文件路径":次数, // App-Mouse-表示鼠标按键次数
  "mouseDistance":98696, // 鼠标移动像素距离,累计
  "tick":1743260182239,	 // 启动时的时间戳,每次启动客户生成唯一
  "updateTime":"20250329230301", // 数据发送时间精确到秒
  // 以下为每分钟的记录数,记录保留10分钟的数据,服务器端超过时间会写入数据
  "MinuteRecords":[
    // 每个节点为每分钟的数据统计,如果是当前的分钟数则需要进行累加
    // 此信息每秒进行更新
    {	
      "Apps":{	
        "执行文件路径":{	
          "Key":23,	// 按键次数
          "Mouse":3	// 鼠标次数
        },	
        "执行文件路径2":{	
          "Key":4,	// 按键次数
          "Mouse":41 // 鼠标次数
        }
      },	
      "Distance":14894,	// 鼠标移动距离
      "KeyCount":27,	// 总键盘按键次数
      "Minute":"202503292256",	// 当前的分钟数
      "MouseCount":48,	// 总鼠标按键次数
      "LastFlag": 1, // 标记是否为最后一个数据,可选项
    }
  ]
}
```

举例:

```javascript
{	
  ".":1,	
  "<#Space":1,	
  "<^c":1,	
  "<^s":1,	
  "<^v":1,	
  "App-Key-C:\\Program Files\\Mozilla Firefox\\firefox.exe":42,	
  "App-Key-D:\\Program Files (x86)\\Notepad++\\notepad++.exe":23,	
  "App-Key-D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":6,	
  "App-Mouse-C:\\Program Files\\Mozilla Firefox\\firefox.exe":61,	
  "App-Mouse-D:\\Program Files (x86)\\Notepad++\\notepad++.exe":22,	
  "App-Mouse-D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":109,	
  "App-Mouse-SysDefault":11,	
  "App-Mouse-桌面":1,	
  "App-Mouse-资源管理器":1,	
  "Backspace":8,	
  "CapsLock":2,	
  "Enter":4,	
  "LButton":96,	
  "LControl":3,	
  "LShift":4,	
  "LWin":1,	
  "MinuteRecords":[	
    {	
      "Apps":{	
        "D:\\Program Files (x86)\\Notepad++\\notepad++.exe":{	
          "Key":23,	
          "Mouse":3	
        },	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":4,	
          "Mouse":41	
        },	
        "SysDefault":{	
          "Key":0,	
          "Mouse":3	
        },	
        "桌面":{	
          "Key":0,	
          "Mouse":1	
        },	
        "资源管理器":{	
          "Key":0,	
          "Mouse":1	
        }	
      },	
      "Distance":14894,	
      "KeyCount":27,	
      "Minute":"202503292256",	
      "MouseCount":48	
    },	
    {	
      "Apps":{	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":0,	
          "Mouse":12	
        }	
      },	
      "Distance":9663,	
      "KeyCount":0,	
      "Minute":"202503292257",	
      "MouseCount":12	
    },	
    {	
      "Apps":{	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":0,	
          "Mouse":16	
        }	
      },	
      "Distance":9972,	
      "KeyCount":0,	
      "Minute":"202503292258",	
      "MouseCount":16	
    },	
    {	
      "Apps":{	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":0,	
          "Mouse":22	
        }	
      },	
      "Distance":8864,	
      "KeyCount":0,	
      "Minute":"202503292259",	
      "MouseCount":22	
    },	
    {	
      "Apps":{	
        "D:\\Program Files (x86)\\Notepad++\\notepad++.exe":{	
          "Key":0,	
          "Mouse":19	
        },	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":0,	
          "Mouse":9	
        },	
        "SysDefault":{	
          "Key":0,	
          "Mouse":6	
        }	
      },	
      "Distance":33492,	
      "KeyCount":0,	
      "Minute":"202503292300",	
      "MouseCount":35	
    },	
    {	
      "Apps":{	
        "C:\\Program Files\\Mozilla Firefox\\firefox.exe":{	
          "Key":42,	
          "Mouse":8	
        },	
        "SysDefault":{	
          "Key":0,	
          "Mouse":1	
        }	
      },	
      "Distance":14272,	
      "KeyCount":42,	
      "Minute":"202503292301",	
      "MouseCount":8	
    },	
    {	
      "Apps":{	
        "C:\\Program Files\\Mozilla Firefox\\firefox.exe":{	
          "Key":0,	
          "Mouse":53	
        },	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":2,	
          "Mouse":7	
        },	
        "SysDefault":{	
          "Key":0,	
          "Mouse":1	
        }	
      },	
      "Distance":7539,	
      "KeyCount":2,	
      "Minute":"202503292302",	
      "MouseCount":61	
    },	
    {	
      "Apps":{	
        "D:\\Program\\Mon\\DebugViewNt\\Dbgview.exe":{	
          "Key":0,	
          "Mouse":2	
        }	
      },	
      "Distance":98696,	
      "KeyCount":71,	
      "Minute":"202503292303",	
      "MouseCount":203	
    }	
  ],	
  "RButton":11,	
  "Space":5,	
  "WheelDown":82,	
  "WheelUp":16,	
  "b":2,	
  "c":2,	
  "d":3,	
  "e":4,	
  "g":3,	
  "h":4,	
  "i":1,	
  "j":1,	
  "k":1,	
  "m":1,	
  "mouseDistance":98696,	
  "r":2,	
  "s":3,	
  "t":2,	
  "tick":1743260182239,	
  "u":2,	
  "updateTime":"20250329230301",	
  "v":2,	
  "w":1,	
  "x":1	
}
```

#### 特殊应用名

| 应用名称                             | 含义                                                 |
| ------------------------------------ | ---------------------------------------------------- |
| NotFoundActive                       | 此时找不到激活的窗口                                 |
| SysDefault                           | 旧的找不到激活窗口时用的名字，新版本1.40之后废弃不用 |
| Desktop                              | 表示为桌面                                           |
| 含有 ApplicationFrameHost.exe:标题名 | 表示可能为UWP应用，后面增加标题以区分                |
| Title:标题名                         | 获取进程路径失败时候用进程标题显示                   |
| Unknown                              | 无法获取进程名和标题                                 |

## WS接口清单

地址和端口号同HTTP

connect链接时传递参数，定义客户端类型(clientType) 清单如下

| clientType（string） | 含义                                                         |
| -------------------- | ------------------------------------------------------------ |
| 空值                 | 每次键盘鼠标触发时发送当前按键汇总信息                       |
| ahkClient            | 在 setPara  时，如果有影响到客户端的参数变化，则发送消息通知客户端重新载入 |



