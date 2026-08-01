# LSRustMiner

LSRustMiner 是一个矿池代理程序。支持套入NUC代理。可配合LSR使用。

## 当前版本重要说明

**当前版本仅 SHA256D 算法完全支持，其它算法未完善，未测试，暂时只建议用于 BTC。**

其余算法仍未完善，尤其是抽水相关功能请谨慎使用。  
如果用于非 BTC / 非 SHA256D 场景，请自行充分测试后再使用。

## 安装脚本
```text
bash <(curl -fsSL https://raw.githubusercontent.com/867897/LSRUSTMINER/main/install.sh)
```
## LSR安装
```text
bash <(curl -fsSL https://raw.githubusercontent.com/867897/LSRUSTMINER/main/LSR.sh)
```
## 默认后台账号

首次启动且没有已有配置时，默认后台账号为：

```text
用户名：admin
密码：admin123
```

请在首次登录后尽快修改默认密码。

## 使用提示

- 当前重点支持 BTC / SHA256D。
- Web 后台登录后会提示当前版本支持范围。
- 提示弹窗支持“下次不再提醒”。
- 非 SHA256D 算法和抽水功能请谨慎开启。

## 免责声明

本程序仅供学习、研究和合法授权环境下使用。使用者应自行确认所在地区法律法规，并对使用行为和结果负责。

## 程序仅收取千分之2作为开发费，可自行测试！！！
