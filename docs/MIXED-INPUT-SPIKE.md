# 五笔/拼音混输 Spike

日期：2026-10-23

目标：验证 RIME 能否在一个 session、一个候选列表里同时提供五笔和拼音候选，作为 Sogou 式混输的落地路线。

## 结论

**单 RIME 组合 schema 可行。**

实测一个 session 同时产生两类候选：

```text
wqiy   → 你、仰泳、武器有、武器、为其
nihao  → 你好、悄虚为、悄虚、你、拟
wq     → 你、仰泳、低调、低、像
abc    → 节能、藂、菆、啊不错、阿布
```

其中：

- `wqiy` 的五笔候选在前
- `abc` 的五笔“节能”在前
- `nihao` 的拼音“你好”在前，因为 RIME 没有把 `nihao` 视为优先的五笔精确码
- `wq` 的五笔候选在前

因此“同一 session 混输”可行，但“五笔精确候选永远第一”还需要额外排序策略。

## 现有 `wubi_pinyin` 为什么不能用

RIME 上游的 `wubi_pinyin` 是：

```text
五笔 + 反引号反查拼音
```

输入 `nihao` 不会同时出现五笔和拼音候选；只有：

```text
`nihao'
```

才查拼音。因此它不是 Sogou 式自动混输。

## 实验 schema

文件：

```text
prototype/rime/wubi_mixed.schema.yaml
```

关键配置：

```yaml
engine:
  translators:
    - punct_translator
    - table_translator
    - script_translator@pinyin

translator:
  initial_quality: 10
  dictionary: wubi86

pinyin:
  initial_quality: 0
  tag: pinyin
  dictionary: pinyin_simp
  prism: pinyin_simp

abc_segmentor:
  extra_tags:
    - pinyin
```

思路：

- `table_translator` 提供五笔
- `script_translator@pinyin` 提供拼音
- `abc_segmentor.extra_tags` 让同一段字母输入同时带 `pinyin` tag
- `initial_quality` 让五笔候选整体排在拼音前

## 实测环境

- 用户数据副本，不修改真实词库
- `rime_api::full_deploy_and_wait()`：`Success`
- schema：`wubi_mixed`
- 使用 `rimeprobe` 风格的 session 查询

## 发现

### 可行

- 一个 session、一个候选列表
- 数字选词、翻页、删词、提交都只需一条路径
- 不需要 Rust 合并两个 session
- 不需要新增候选窗协议

### 仍需解决

1. **五笔精确候选的硬优先级**
   - `initial_quality` 只能整体加权
   - `nihao` 仍是拼音候选在前
   - 可能需要 `filters` 或自定义排序

2. **候选来源识别**
   - 当前候选只有文本；用户无法知道是五笔还是拼音
   - 是否显示来源/注释需要 UI 决策

3. **schema 安装与升级**
   - 新 schema 必须进入 `schema_list` 才会被部署
   - 老用户已有 `default.custom.yaml`，升级时需要幂等迁移

4. **配置开关**
   - 建议新增 `mixed_input`，不复用 `pinyin_fallback`
   - 默认值需要决定：保持旧行为还是默认开启

5. **纯五笔模式**
   - 用户明确不要纯拼音模式
   - 是否保留纯五笔作为兼容开关，需要另定

## 推荐落地路线

```text
一个 wubi_mixed session
+ config.mixed_input
+ 现有 InputSession / CandidateWindow
```

不采用：

```text
Rust 合并 wubi86 + pinyin_simp 两个 session
```

原因：候选来源、页码、提交、删词、用户词频都会分裂。

## 下一步

1. 决定 `mixed_input` 默认值和纯五笔兼容策略
2. 决定五笔精确优先的排序实现
3. 把 schema 加入安装/升级流程
4. 再把 engine 从“双 session”切到“单 mixed session”
