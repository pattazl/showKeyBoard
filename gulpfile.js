const gulp = require('gulp');
const del = require('del');
const rename = require('gulp-rename');
const es = require('event-stream');
const nls = require('vscode-nls-dev');
const ts = require('gulp-typescript');
const sourcemaps = require('gulp-sourcemaps');
const typescript = require('typescript');

const tsProject = ts.createProject('./tsconfig.json', { typescript });
const inlineMap = true;
const inlineSource = false;

// 支持的语言
const languages = [{ folderName: 'zh-cn', id: 'zh-cn' }];

const cleanTask = function() {
        return del(['out/**', 'package.nls.*.json']);
}

// 源码
const sourcesNsl = function() {
        var r = tsProject.src()
                .pipe(sourcemaps.init())
                .pipe(tsProject()).js
                .pipe(nls.rewriteLocalizeCalls())
                .pipe(nls.createAdditionalLanguageFiles(languages, 'i18n', ''))
                .pipe(nls.bundleMetaDataFiles('austin-ts-nls', 'out'))
                .pipe(nls.bundleLanguageFiles());

        // sourcemap
        if (inlineMap && inlineSource) {
                r = r.pipe(sourcemaps.write());
        } else {
                r = r.pipe(sourcemaps.write("../out", {
                        // no inlined source
                        includeContent: inlineSource,
                        // Return relative source map root directories per file.
                        sourceRoot: "../src"
                }));
        }
        // 输出到out目录
        return r.pipe(gulp.dest("out"));
};

// package.json
const packageNls = function() {
        return gulp.src(['package.nls.json'], {allowEmpty: true})
                .pipe(nls.createAdditionalLanguageFiles(languages, 'i18n'))
                .pipe(gulp.dest('.'));
};
// clipboard scripts
const clipboard = function() {
        return gulp.src('src/clipboard/*')
                .pipe(gulp.dest('out/clipboard'));
};
const nlsTask = gulp.series(cleanTask, sourcesNsl, packageNls, clipboard);

gulp.task('clean', cleanTask);

gulp.task('nls', nlsTask);

// 提取需要翻译的字符串到i18n/base目录，方便翻译
const sourcesMsg = function () {
        const suffix = '.i18n.json';
        var r = tsProject.src()
                .pipe(sourcemaps.init())
                .pipe(tsProject()).js
                .pipe(nls.rewriteLocalizeCalls())
                .pipe(nls.createKeyValuePairFile())
                .pipe(es.through(function (file) {
                        // 仅处理.i18n.json
                        if (file.path.indexOf(suffix, file.path.length - suffix.length) !== -1) {
                                this.queue(file);
                        }
                }))
                .pipe(gulp.dest(`./i18n/base`));
        return r;
};

// package.nls.json，结构一致，只需要拷贝一份
const packageMsg = function () {
        var r = gulp.src(['package.nls.json'])
                .pipe(rename({ basename:"package", suffix: ".i18n"}))
                .pipe(gulp.dest(`./i18n/base`));
        return r;
};

const messageTask = gulp.series(sourcesMsg, packageMsg);
gulp.task('message', messageTask);