// 获取全部参数信息
var hashPara = {}
async function getPara() {
	const requestOptions = {
	  method: 'POST',
	  headers: {
		'Content-Type': 'application/json',
	  }
	};
    let rsp = await fetch("/getPara",requestOptions)
	hashPara = await rsp.json();
	info.value = ( JSON.stringify(hashPara))
}

getPara()


async function setPara() {
	
	let hash = JSON.parse(info.value)
	const requestOptions = {
	  method: 'POST',
	  headers: {
		'Content-Type': 'application/json',
	  },
	  body: JSON.stringify(hash),
	};
    let rsp = await fetch("/setPara",requestOptions)
	let result = await rsp.json();
	console.log(result)
}
