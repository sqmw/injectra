# 优酷 CDP容器profile参考

历史版本9.2.73；[后续排查证据](../../../../docs/compatibility/macos-cdp-followup.md)记录沙箱外profile失败、移入自身容器缓存后基线载荷通过。[元数据](case.json)及launch.example.sh仅归档这些启动条件，本次没有重新操作优酷。

显式设置APP_EXECUTABLE、CDP_PORT和新的RUN_DIR。RUN_DIR必须位于当前用户的Library/Containers/com.youku.mac/Data/Library/Caches下；不要把任意外部profile路径套到此版本。脚本只接受此位置模式，不删除既有目录。

入口可达后，使用[公共CDP CLI样例](../../../shared/cdp/README.md)选择精确页面并提交可信载荷。这里仅隔离新Chromium profile，不承诺全部宿主偏好和业务缓存隔离。
