# 共享Web程序生成器参考入口

小型独立Rust crate，调用产品现有injectra-web的Bundle::load/apply/remove；不复制web/runtime.js。路径依赖从本目录推导到主仓库crates/web。

以自己的空workspace与Cargo.lock隔离，不加入产品workspace。`make -C references program`显式构建到仓库外reference cache；不会由根make build/check隐式构建。

来源为WK探针db3a65e的小型生成器入口，仅调整包名、路径及产物位置。由[抖音案例](../../apps/douyin/gadget-v8/README.md)使用，离线载荷检查不构成真实应用兼容性证明。
