# Remote Shutdown - 远程关机

基于 TCP/MQTT 实现的远程关机的程序。

本人使用了基于巴法云物联网平台（ bemfa.com ）的物联网平台进行测试，理论上本应用支持所有采用了类似协议的物联网平台使用。

在实现上主要参考了：

> https://blog.csdn.net/qq_28997215/article/details/122604695

## 使用方法

### 准备程序及配置文件

#### 自行编译

安装好 Rust 环境后，克隆本项目代码并在本地运行：

```shell
git clone https://github.com/bearbattle/remote-shutdown.git
cd remote-shutdown
cargo build --release
cargo run --release
```

#### 二进制分发

你也可以选择在 Release 页面下载编译好的二进制版本，这些二进制版本均是静态链接的，可以直接运行而不需要额外的运行库：

https://github.com/bearbattle/remote-shutdown/releases/latest

请下载与你所使用的平台对应的程序版本。

配置文件分布在仓库的根目录中，你也可以在 Release 下载的压缩包中找到对应的配置文件示例。

### 平台准备

前往巴法云平台（https://bemfa.com/）注册账号，之后在 `TCP 设备云`或者 `MQTT 设备云` 新建主题。

建议使用 MQTT 设备云创建主题，MQTT 原生支持断线重连，TCP 设备的断线重连是作者自己实现的，不要太相信我。

> [!IMPORTANT]
> 请注意创建的主题名称！
> 
> 当主题名字后三位是001时是插座设备
> 
> https://cloud.bemfa.com/docs/src/index_device.html

创建完主题后，点击 `更多设置` ，在弹出的 `数据存储收费` 页面点击取消。

在 `设备云` 下的文本框中输入新的设备昵称，例如 “电脑”，点击 `更新昵称` 。

（很重要，之后会使用这个昵称识别设备）

然后打开米家APP，打开右下角 `我的` ，在列表中找到 `添加其他平台设备`，点击 `添加` 按钮 ，找到 `巴法云` 。

找到之后需要输入账号及密码绑定巴法云账号。

点击同步设备，可以看到已经添加的设备的昵称。

### 编辑配置文件

找到自己使用的配置文件模板。例如，如果使用的是 `TCP 设备云` 创建的主题，就使用 `config.toml.example-tcp` ；如果使用的是 `MQTT 设备云` 创建的主题，则可以使用 `config.toml.example-mqtt` 。

如果你对安全性有较高要求，可以考虑使用 `config.toml.example-mqtt-tls` 或 `config.toml.example-wss` 作为你的配置文件模板。

将找到的配置文件模板重命名，去掉最后一截扩展名，例如将 `config.toml.example-tcp` 重命名为 `config.toml` 。

在 `config.toml` 中填写对应的信息：

```toml
proto = "mqtt"

[config]
host = "bemfa.com"
port = 9501
topic = "IPASION001" # 自行添加的主题名称
uid = "fexxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # 可以在巴法云控制台首页找到私钥，填入此处
```

详细配置可以参考巴法云接入文档：
- https://cloud.bemfa.com/docs/src/tcp.html
- https://cloud.bemfa.com/docs/src/mqtt.html

保存文件，将这个文件和程序放在一起。

### 运行程序

直接点击运行，程序将注册对应的设备并开始监听消息。

此时使用小爱同学对她说关掉对应的设备即可完成操作。