# chat

## chat_core

```shell
cd chat_core/fixtures
# 生产ed25519的私钥与公钥
openssl genpkey -algorithm ed25519 -out encoding.pem
openssl pkey -in encoding.pem -pubout -out decoding.pem
```
