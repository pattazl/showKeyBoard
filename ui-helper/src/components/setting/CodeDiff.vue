<template>
  <div>
    <div @click="diffHtmlClicked" v-html="diffHtml"></div>
  </div>
</template>

<script>
import { createPatch } from 'diff'
import { parse, html } from 'diff2html'
import 'diff2html/bundles/css/diff2html.min.css'

export default {
  name: "CodeDiff",
  props: {
    // 原文本内容
    oldString: {
      type: String,
      default: ''
    },
    // 新文本内容
    newString: {
      type: String,
      default: ''
    },
    // 以差异的文本为基准，上下展示的文本行数
    context: {
      type: Number,
      default: 5
    },
    // 展示方式：line-by-line || side-by-side
    outputFormat: {
      type: String,
      default: 'line-by-line'
    },
    // 是否展示文件列表
    drawFileList: {
      type: Boolean,
      default: false
    },
    // 切换文件内容状态，在显示与隐藏之间切换
    fileContentToggle: {
      type: Boolean,
      default: true
    },
    renderNothingWhenEmpty: {
      type: Boolean,
      default: false
    },
    // 文件名
    fileName: {
      type: String,
      default: ''
    },
    // 如果没有差异，是否展示原文
    showTextWithoutDiff: {
      type: Boolean,
      default: true
    }
  },
  computed: {
    diffHtml() {
      return this.createdHtml(this.oldString, this.newString, this.context, this.outputFormat, this.drawFileList, this.fileContentToggle, this.renderNothingWhenEmpty, this.fileName, this.showTextWithoutDiff)
    }
  },
  mounted(){
     let arr = document.getElementsByName('viewed')
     console.log(arr)
     for(let v of arr)
     {
      if(!v.checked)v.click();
     }
  },
  methods: {
    createdHtml (oldString, newString, context, outputFormat, drawFileList, fileContentToggle, renderNothingWhenEmpty, fileName, showTextWithoutDiff) {
      // 获取本文相似度
      // 设置diff.js的createPatch方法的入参
      let args = [fileName, oldString, newString, '', '', { context: context }]
      // diff.js的createPatch()方法，输出oldString和newString差异补丁，内容是GUN DIFF格式
      let diffString = createPatch(...args)
      // 如果两份文件没有差异，createPatch输出的结果，不会带有原文本信息，下面是强制加上原文本信息
      console.log(oldString)
      console.log(newString)
      console.log(diffString)
      // 将diffString转换为DiffFile[]
      // let diffJson = parse(diffString, {
      //   outputFormat,
      //   drawFileList,
      //   renderNothingWhenEmpty,
      //   matching: 'lines'
      // })
      // 将DiffFile[]转换为html标签
      let diffHtml = html(diffString, {
        outputFormat,
        drawFileList,
        renderNothingWhenEmpty,
        //fileListStartVisible:'true',
        matching: 'none', // lines
        // 自定义展示方式
        rawTemplates: {
          //自定义标题栏
          "generic-file-path": `<span class="d2h-file-name-wrapper">
    {{>fileIcon}}
    <span class="d2h-file-name">{{fileDiffName}}</span>
    <span class="d2h-tag d2h-changed d2h-changed-tag">差异数</span>
    <span class="d2h-diff-title" style="display: none; flex: 1 1 auto; text-align: center">修改前</span>
    <span class="d2h-diff-title" style="display: none; flex: 1 1 auto; text-align: center">修改后</span>
</span>
<label class="d2h-file-collapse" style="display: flex;;width:60px">
    <input class="d2h-file-collapse-input" type="checkbox" name="viewed" value="viewed">
    Viewed
</label>`
        }
      })
      // 添加d2h-d-none类，默认隐藏diff内容
      diffHtml = diffHtml.replace('d2h-files-diff', 'd2h-files-diff d2h-d-none')
      return diffHtml
    },
    /**
     * 监听v-html的点击事件，这里主要是为了点击“VIEWED”元素后，diff内容能显示和隐藏
     * @param e
     */
    diffHtmlClicked(e) {
      // 如果点击的元素是INPUT，说明是“VIEWED”元素，用于显示和隐藏diff内容
      if(e.target.nodeName === 'INPUT') {
        // 获取点击的“VIEWED”元素
        let viewedElement = e.target
        // 定位到header区
        let d2hHeader = viewedElement.parentNode.parentNode
        // 获取标题区
        let d2hDiffTitle = d2hHeader.querySelectorAll('.d2h-diff-title')
        // 标题的显示与隐藏
        d2hDiffTitle.forEach((item) => {
          if(item.style.display == 'none') {
            item.style.display = 'block'
          } else {
            item.style.display = 'none'
          }
        })
        // 定位到对应的diff内容元素
        let diffText = viewedElement.parentNode.parentNode.nextElementSibling
        // 如果元素只有d2h-files-diff一个class，说明当前是展开状态
        // 如果元素有d2h-files-diff d2h-d-none两个class，说明是隐藏状态
        if(diffText.classList.value === 'd2h-files-diff') {
          diffText.classList.add('d2h-d-none')
        } else {
          diffText.classList.remove('d2h-d-none')
        }
      }
    }
  }
}
</script>

<style scoped>
.d2h-file-name-wrapper {
  display: flex
}
</style>