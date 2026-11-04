# 现有CDP CLI调用样例

[apply.example.sh](apply.example.sh)只调用现有Injectra CLI，不是另一套CDP实现。执行前显式设置INJECTRA_BIN、CDP_PORT、TARGET_ID及BUNDLE_MANIFEST。

先用现有`targets --port`检查页面身份，再指定精确TARGET_ID；不自动选择第一个页面，不自动启动/结束应用。Bundle使用调用者信任的本地manifest。启动条件分别见[Figma](../../apps/figma/cdp/README.md)、[优酷](../../apps/youku/cdp/README.md)、[WorkBuddy](../../apps/workbuddy/cdp/README.md)。
