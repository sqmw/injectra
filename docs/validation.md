# 验收入口

这里仅定位证据，不重复维护应用状态或历史测试计数。

| 要核对什么 | 权威入口 |
| --- | --- |
| 当前各应用、平台、内核及路线结果 | [兼容性总表](compatibility/current-matrix.md) |
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
