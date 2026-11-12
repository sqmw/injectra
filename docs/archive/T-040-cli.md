# T-040 · CLI收敛与dev闭环

<a id="t-040"></a>

[当前Task](../TODO.md#t-040) · [验收](../validation/cli-dev.md)

```tracking-execution
{
  "id": "T-040/E-002",
  "name": "CLI 收敛与 dev 闭环",
  "task_id": "T-040",
  "task_version": "v3",
  "status": "succeeded",
  "authorization": "2026-11-12用户明确PLEASE IMPLEMENT THIS PLAN，批准选择性删除平台化并打通CLI dev；页面增量、直接应用、无startup。",
  "scope": "删除工作台、Studio、完整workspace及启动增量/导出；提取Bootstrap、最小来源及执行正确性；精简准备器，只改派生副本，保留历史和运行数据。",
  "steps": [],
  "evidence": [
    "删除前main及4个platform工作树源码快照、HEAD、patch保存在仓库外recovery/t040-cli-20261112；不改其他工作树。已移除平台模块，提取application dev输入版本及Bootstrap请求绑定；cargo check通过。独立prepare/dev与最小未决记录已实现，make check、独立Electron、38项Frida测试、目录/ASAR两类dev夹具及Claude本地壳实机通过；QQ当前CJS入口Unsupported如实保留。",
    "a403982选择性删除Studio、前端、workspace；323db43实现独立CLI prepare/dev。Parent审查原runtime/core/native/后端源码与4cf736e无差异，未整段回退或重写历史。",
    "最终make check、独立Electron smoke、38项Frida测试、Bootstrap三项JS测试、core/native独立构建、doc-check/tracking-check/diff-check通过。目录/ASAR CLI dev双窗口隔离、连续更新、预检/入口身份/相对解析/strict、源码位置、页面重建停止、资源摘要、ESM修改前拒绝均通过。",
    "Claude4.23118.1真实本地壳页面通过两版JS/HTML/CSS断言和清理回执，关键资源摘要未变；Ctrl-C保留开发实例。console事件未取得，未宣称日志或远端聊天通过。QQ6.9.86因主入口非普通可读CJS而Unsupported；没有按品牌绕过。",
    "仅验证所创建的副本PID经路径核对后退出；用户原实例、账号业务和旧实验未操作。外部测试增量及证据不进入仓库或插件库。",
    "当前server工作树已通过11fc3a1合入CLI收敛；源码与main一致，Task只读快照。其他三个平台树的未提交工作保留。文档同步范围、CLI、依赖/源码导航、兼容表和恢复映射。"
  ],
  "stop_reason": ""
}
```
