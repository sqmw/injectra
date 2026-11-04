# 抖音 Gadget → V8/Node → 页面参考

历史来源为独立探针622fe2e，[文件来源](origin.json)、[案例元数据和统计归属](case.json)。原8.7.0真实页面及13项通过的范围、7.9失败/版本漂移/截图旧帧见[主报告](../../../../docs/compatibility/douyin-gadget.md)。本次路径迁移未重新操作抖音，不能把历史13项当作迁移版新实机结果。

## 源码结构

launch.py与apply_page.py保留软件bundle ID、单实例条件和精选页目标选择。参考路径由reference_paths.py推导；Gadget、V8与Node/Page bridge只在shared/electron-v8保留一份。其余脚本和payload为本次验证辅助，按case.json单独统计。

## 人工复核入口

```sh
make -C references setup program
make -C references gadget
# 先经用户授权正常退出原抖音；脚本不会替用户退出。
make -C references/apps/douyin/gadget-v8 launch APP_PATH="安装位置/抖音.app" PROXY_URL="当前代理URL"
make -C references/apps/douyin/gadget-v8 node pages
make -C references/apps/douyin/gadget-v8 apply capture
# 实际点击一次面板按钮、确认计数1后：
make -C references/apps/douyin/gadget-v8 verify capture
```

Launch默认required模式及LaunchServices，拒绝已有实例，按真实PID/profile核对后推进。Gadget依赖从公共reference cache读取；本案例run默认独立放在公共cache的runs/douyin-gadget-v8，不使用旧Probe目录。INJECTRA_GADGET_CACHE仅作为显式run路径覆盖。

页面建立可能晚于启动回执，先检查pages再apply；不新建假页面冒充真实应用。recover_bootstrap.py只用于核对本次已拥有的延迟启动，不能覆盖旧超时。verify要求初始点击恰为1；多次交互后不要为重跑清空用户正在使用的状态。

源码仍有特定平台/符号/ABI及资源边界，不是产品WebBackend。原受控run的源码哈希继续指向原Probe；本次变化是路径/依赖适配、生成器独立及源码哈希定位修正，未声称主程序/页面业务的跨版本兼容。
