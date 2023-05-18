# aptos-codecat
一个用于GitHub收藏管理的dapp。
### 运行
#### env
```
export PRIVATE_KEY=xxx 
export MODULE_ADDRESS=0x2d81030a4f7d00975e96f823bad1a72b235de421d1ea7ded523105fde8113421
```
#### compile
```
cargo build
```
#### use codecat
```
// query the content of the uri with key "test".
./target/debug/codecat --name test

// add the uri corresponding to the code.
./target/debug/codecat --name test --uri https://github.com/jjwygjj/aptos-codecat.git
```
