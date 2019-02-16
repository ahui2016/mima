## 先用一个有权限创建角色的身份登入数据库，创建一个名为 mimauser 的角色。

```sql
CREATE ROLE mimauser LOGIN CREATEDB PASSWORD 'abc';
```

## 再以 mimauser 的身份登入数据库，创建一个名为 mimadb 的数据库。

```sql
CREATE DATABASE mimadb;
```

## 再以超级用户身份连接到 mimadb, 创建所需的扩展。

```sql
CREATE EXTENSION pgcrypto;
```

## 登出数据库。至此，数据库准备完毕。接下来，使用 diesel_cli 创建数据表。（若未安装 diesel_cli, 请现在安装）

## 在项目根目录新建一个文件 .env, 内容如下：

```
DATABASE_URL=postgres://mimauser:abc@localhost/mimadb
ROCKET_DATABASES={db_todo={url="postgres://mimauser:abc@localhost/mimadb"}}
```

## 执行 `diesel setup`

## 执行 `diesel migration generate create_tables`, 该命令会生成文件 up.sql 和 down.sql.

## 在文件 up.sql 和 down.sql 中输入内容。

## 执行 `diesel migration run`

## 至此，所需的数据表也已准备完毕。关于数据库的一切准备工作已完成。
