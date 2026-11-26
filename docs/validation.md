# 验收入口

当前范围见[dev模式](architecture/dev-mode.md)。T-040 v1工作台局部测试、CDP和编辑保存检查只作留存证据，不等于v3通用dev能力验收；旧记录不改写。


这里仅定位证据，不重复维护应用状态或历史测试计数。

最新批次：[T-032十款macOS应用顺序验证](validation/macos-ten-apps.md)，五款已取得页面合同成功证据，五款接入未完成；含获授权后阿里6.9.3补测，未将批次完成误报为全部应用支持。

后续路线：[T-033阿里云盘Frida验证](compatibility/adrive-frida.md)，Gadget加载未取得身份，直接附加被拒绝；Hook/页面未执行，清理与普通窗口恢复已核对。

早期入口探索：[T-034阿里云盘pre-runtime核查](compatibility/adrive-preruntime.md)，只读候选证据与缺失接口分别记录，尚无新增实机注入成功。

T-034继续：[启动观察器与自测](validation/startup-observer.md) · [阿里启动条件实测](compatibility/adrive-startup.md)。进程交接、HTTP原始回执与授权等待分别记录，观察器测试不作为目标应用注入成功。

| 要核对什么 | 权威入口 |
| --- | --- |
| 当前各应用、平台、内核及路线结果 | [兼容性总表](compatibility/current-matrix.md) |
| M-003 代表商业宿主正式 CLI 实证回测（T-088） | [商业宿主正式 CLI 回测](validation/m3-commercial-cli.md) |
| 长期架构重构的产品回归与独立构建 | [架构验收](validation/architecture-foundation.md) |
| MVP用户验收和各批次原始摘要 | [历史验收正文](archive/validation-history.md) |
| 版本与源码对应 | [交付映射](releases.md) |
| 参考源码的检查边界和分组统计 | [参考实现](../references/README.md) |

当前检查入口与使用方法见[开发说明](dev.md)。实测只覆盖记录中的版本、页面及载荷；契约替身、代码统计和文档校验不等于真实兼容性证明。

T-017新增Mac后端与宿主验收见[Mac交付](macos-delivery.md)、[Frida](frida-backends.md)、[CEF/NW.js](host-validation.md)、[WebKit](webkit-backend.md)；不覆盖原MVP与探针历史证据。

本轮集中[验收与自审](validation/macos-multi-engine.md)。

T-019控制端/设备解耦的[验收与自审](validation/device-boundaries.md)，包括保留的失败和后续验证。

T-021：[Firefox/Gecko实机验证](gecko-validation.md)，独立测试路线与正式CLI支持分别记录。

T-022：[Thunderbird安装与验收](thunderbird-validation.md)，主界面DOM路线和内容页通过，HTML字符串限制与整组partial如实保留。

T-024：[Thunderbird主界面HTML装配路线](validation/thunderbird-html.md)，新路线通过，原innerHTML合同仍partial；正式CLI另行推进。

T-025：[抖音原生Interceptor新路线验证](compatibility/douyin-native-hook.md)，原包前置接入受限；自有进程Hook成功不外推为抖音支持。

T-026：[抖音Objective-C分派Hook新路线](compatibility/douyin-objc-hook.md)，原包内自有测试对象的Hook/恢复通过；不等同于Interceptor或业务方法兼容。

T-027：[抖音Interceptor授权重试](compatibility/douyin-hook-auth-retry.md)：原包两条路线仍未通过，新增外部attach控制成功；用户未见首次重试授权窗口，未修改系统保护。

T-028：[抖音真实宿主方法Hook](compatibility/douyin-host-hook.md)：AtomApplication.sendEvent:真实调用及原实现转发通过；与测试对象、业务逻辑和Interceptor区分。

T-029：[Hook触发原抖音主题预览](archive/T-029.md#t-029)，真实页面紫雾外观可见并保留窗口；Hook触发与既有Web桥接分工明确，CLI Unknown及悬挂主导航处置独立记录。
