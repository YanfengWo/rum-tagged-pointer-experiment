# Next Session Handoff

这份文件是给新会话快速接管当前仓库状态用的。  
如果新会话发现上下文很长、时间线不清，**先看这份文件，再看 `git log` / `git show`，不要只靠会话记忆复原状态。**

## 当前状态

- 仓库路径：`/home/fft/src/rum`
- 当前分支：`main`
- 当前状态：本地开发仓库，`origin` 已移除，VS Code 里看到的“发布 branch”可以忽略
- 工作区状态：本文件提交后应保持干净

## 最近关键提交

- `9551e12` `Close CL-07 deep parity wave`
- `ade3fad` `Add DeleteCases and richer association lookup parity`
- `9427ecd` `Finish CL-07 string mainline`
- `8483a33` `Finish CL-09 debugging and verification mainline`
- `dc6fbbd` `Finish CL-08 package tooling mainline`

如果要复原最近几步的真实时间线，优先用：

```bash
git log --oneline --decorate -12
git show --stat -1 <commit>
git diff --stat HEAD~1..HEAD
```

## 主线进度概览

### 已完成主线

- `CL-07`：主线完成，当前只剩 non-blocking polish
- `CL-08`：主线完成
- `CL-09`：主线完成

### 当前 roadmap 所处阶段

- `TODO.md` 里的 **Post-Mainline Long-Tail Priorities**
- `Wave 1: CL-07 deep parity` 已完成
- 当前应进入：
  - `Wave 2: CL-03 and CL-05`

## 已经落地的 Wave 2 起点

### CL-03

已做：

- `DeleteCases` first-pass
  - 默认 level `{1}`
  - 支持 `levelspec`
  - 支持第四参 `n`
  - 支持 `Heads -> True|False`
  - `Association` 中按 value 删除整条 entry

对应提交：

- `ade3fad`

关键文件：

- [rewrite_builtins.rs](/home/fft/src/rum/crates/rum-runtime/src/runtime/rewrite_builtins.rs)
- [definitions_and_rules_part3.rs](/home/fft/src/rum/crates/rum-runtime/src/runtime/tests/definitions_and_rules_part3.rs)

### CL-05

已做：

- richer `Lookup` parity
  - multi-key lookup
  - list-of-associations threading
  - 和已有 key-path 语义兼容

对应提交：

- `ade3fad`

关键文件：

- [basic.rs](/home/fft/src/rum/crates/rum-runtime/src/runtime/associations/basic.rs)
- [associations.rs](/home/fft/src/rum/crates/rum-runtime/src/runtime/tests/associations.rs)

## 下一步建议

如果继续当前主线，建议优先顺序：

1. `CL-05`
   - richer `Missing` semantics
   - `KeySortBy` option/comparer parity
   - broader keyed-data compatibility polish
2. `CL-03`
   - `ReplaceList`
   - `ReplaceAt`
3. `CL-03` deeper parity
   - `PatternSequence`
   - `OrderlessPatternSequence`
   - `Verbatim`
   - deeper traversal parity

当前已补上的 Wave 2 关键切口：

- `DeleteCases`
- richer `Lookup` parity
- `Replace` first-pass
- `AssociateTo` / `KeyDropFrom` held mutation parity
- shared raw-option lookup cleanup for `Dialog` / `Monitor` / `TestReport`
- shared no-limit traversal normalization reuse for `Count` / `MemberQ` / `FreeQ`

## 关键文档

新会话开始时建议优先看这些：

1. [TODO.md](/home/fft/src/rum/TODO.md)
   - 当前路线图与波次顺序
2. [core-language-support-matrix.md](/home/fft/src/rum/docs/core-language-support-matrix.md)
   - 各 `CL-*` 的当前口径
3. [code-organization-guidelines.md](/home/fft/src/rum/docs/code-organization-guidelines.md)
   - 文件拆分原则、体积阈值、测试对齐原则
4. [ARCHITECTURE.md](/home/fft/src/rum/ARCHITECTURE.md)
   - 体系边界与 crate 职责
5. 如需 host/adapters：
   - [host-adapter-plan.md](/home/fft/src/rum/docs/host-adapter-plan.md)
   - [python-adapter-plan.md](/home/fft/src/rum/docs/python-adapter-plan.md)

## 工作原则

### 1. 先看 git，再动手

如果会话已经很长，或者感觉 assistant 出现“记忆漂移”，务必：

```bash
git log --oneline --decorate -12
git show --stat -1 HEAD
git status --short
```

### 2. 小步提交

- 一次只收一个清晰语义切片
- 不把不相干的工作混进同一提交

### 3. 做完就补测试

至少：

```bash
cargo test -p rum-runtime
```

最好再跑对应定向测试，例如：

```bash
cargo test -p rum-runtime definitions_and_rules_part3
cargo test -p rum-runtime associations
```

### 4. 文档口径同步

当某条主线或 wave 的状态发生实质变化时，同步：

- [TODO.md](/home/fft/src/rum/TODO.md)
- [core-language-support-matrix.md](/home/fft/src/rum/docs/core-language-support-matrix.md)

### 5. 遵守代码组织规则

按 [code-organization-guidelines.md](/home/fft/src/rum/docs/code-organization-guidelines.md)：

- 按职责拆，不按行数硬切
- 测试尽量跟实现结构对齐
- `20k+` 是 split-plan 区
- `35k+` 是明确架构债

## 额外提醒

- `WLBuiltin/` 目录已经在 `.gitignore` 中
- 它是下一阶段候选输入，不要在当前主线里顺手卷进来
- 除非用户明确要求，否则先继续 `CL-03 / CL-05`

## 推荐开场提示

如果新会话要无缝接手，可以直接从这句话开始：

> 先按 `docs/next-session-handoff.md` 和最近几条 git 记录对齐状态；当前 `CL-07/08/09` 主线已完成，`Wave 2` 已经推进到 `DeleteCases`、richer `Lookup`、`Replace` first-pass 和 held `AssociateTo` / `KeyDropFrom`，下一步优先补 `CL-05` 的 `Missing` / `KeySortBy`，再回到 `ReplaceList` / `ReplaceAt`。
