# WorkBuddy CDP宿主目录参考

历史版本5.9.3；[后续排查证据](../../../../docs/compatibility/macos-cdp-followup.md)记录自身路径环境变量覆盖通用目录，指定后基线载荷通过。[元数据](case.json)和launch.example.sh仅归档该条件，本次没有重新操作WorkBuddy。

显式提供APP_EXECUTABLE、CDP_PORT及仓库外的新RUN_DIR，脚本分别设置WORKBUDDY_CONFIG_DIR和WORKBUDDY_USER_DATA_DIR。不要只依赖--user-data-dir推断全部配置已经隔离。

页面接入与注入仍使用现有CLI及[公共CDP调用样例](../../../shared/cdp/README.md)，不复制一套WorkBuddy专用后端，也不包含业务界面的选择器或重渲染兼容。
