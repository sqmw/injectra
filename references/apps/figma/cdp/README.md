# Figma CDP启动条件参考

历史版本126.5.6，证据来自[兼容报告](../../../../docs/compatibility/figma.md)。[元数据](case.json)记录来源；launch.example.sh根据已记录条件整理，本次只做静态检查，未重新启动Figma。

明确提供APP_EXECUTABLE、独立TEST_HOME及CDP_PORT后，手动执行脚本。FIGMA_TEST=1让当前版本跳过移除调试端口；该内部开关也会影响关闭确认等行为，不能用于原登录profile或当作所有版本稳定API。HOME/CFFIXED_USER_HOME只改变本次子进程，隔离范围仍以读回路径为准。

需要时显式提供PROXY_SERVER；不硬编码本机代理端口。启动后用已有CLI发现并选择页面，再调用[公共CDP样例](../../../shared/cdp/README.md)。历史验证的是38px高的shell.html标签栏，载荷需适合该视口；编辑器未验证，不自动替用户适配界面。
