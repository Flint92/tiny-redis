# tiny-redis

学习RESP以及实现一个简单的redis


## 启动tiny-redis

```shell
cd ./server && RUST_LOG=info cargo run
```

## 使用redis-cli连接tiny-redis

```shell
redis-cli -p 16379
```


### 实现的命令ping

```
127.0.0.1:16379> ping
PONG
127.0.0.1:16379> 
```


### 实现的命令get和set

```
127.0.0.1:16379> set lang rust
"OK"
127.0.0.1:16379> get lang
"rust"
127.0.0.1:16379> get lang2
(nil)
127.0.0.1:16379> 
127.0.0.1:16379> set lang rust-new
"OK"
127.0.0.1:16379> get lang
"rust-new"
127.0.0.1:16379> 
```

### 实现的命令lpush和lrange

```
127.0.0.1:16379> lpush lang2 golang rust python
(integer) 3
127.0.0.1:16379> 
127.0.0.1:16379> 
127.0.0.1:16379> lrange lang2 0 -1
1) "python"
2) "rust"
3) "golang"
127.0.0.1:16379> 
```

### 实现的命令rpush和lrange

```
127.0.0.1:16379> rpush lang3 golang rust python
(integer) 3
127.0.0.1:16379> 
127.0.0.1:16379> 
127.0.0.1:16379> lrange lang3 0 -1
1) "golang"
2) "rust"
3) "python"
127.0.0.1:16379> 

```