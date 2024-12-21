# 验证步骤
执行一下命令，查看是否能够正常输出
```shell
cargo test --test test --  --show-output
```
# 主要功能
1. 设备linkup
2. ring结构建立和初始化
3. 接收队列和发送队列初始化
4. 启用接收和发送相关中断
5. 接收和发送数据
# 备注
有一些接口需要操作系统实现，但我还没适配到arcros上。test.rs
中只是做了简单的测试，具体的实现还需要操作系统的支持。
除了sleep接口外，还有dma_api中的Impl trait和lib文件中的定义
的phy_to_virt需要操作系统支持。
