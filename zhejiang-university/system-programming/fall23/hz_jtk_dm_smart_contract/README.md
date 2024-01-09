# medicalContract-near
该项目是基于near公链使用rust语言编写的医疗场景智能合约，由黄哲&蒋廷恺&段苗合作完成，可访问[源仓库地址](https://github.com/rust-work-medical/medicalContract-near)查看最新版本。
## Quickstart
### 1. Install Dependencies
```bash
npm install
```

### 2. Test the Contract
Deploy your contract in a sandbox and simulate interactions from users.

```bash
npm test
```
### 3. Deploy the Contract
Build the contract and deploy it in a testnet account
```bash
npm run deploy
```

## TroubleShooting
### 1.When you run */.sh file and get this error : "/bin/bash^M:bad interpreter:No such file or directory"
[Check this solution](https://blog.csdn.net/weixin_42891455/article/details/118707204)
### 2.RPC error:"Retrying HTTP request for https://rpc.testnet.near.org because of error: FetchError: request to https://rpc.testnet.near.org/ failed, reason: Client network socket disconnected before secure TLS connection was established"
You should run this two commands orderly:
```bash
export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://hk.bsngate.com/api/1e7227ced74cdd9507575cf08bef7d1ff715db9c8d9befd945b766a86c2ea1fd/Near-Testnet/rpc
```
```bash
near set-api-key https://hk.bsngate.com/api/1e7227ced74cdd9507575cf08bef7d1ff715db9c8d9befd945b766a86c2ea1fd/Near-Testnet/rpc 29e93a93a9868bb25fadf2f5cf19848ca87b31797f963b314b462cbb79dc32ea
```