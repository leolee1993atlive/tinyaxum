# ---------- runtime ----------
# 注意:二进制在宿主机编译(build-docker.sh),动态链接宿主的 glibc。
# 运行镜像的 glibc 必须 >= 编译机,否则启动时报 `GLIBC_x.y not found` 崩溃重启。
# 故此处与编译机(Ubuntu 24.04 / glibc 2.39)对齐,使用 ubuntu:24.04。
# 换编译机时务必同步这里的基础镜像。
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3t64 libstdc++6 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY tinyaxum-rs/target/release/tinyaxum /app/tinyaxum
COPY tinyaxum-ui/build                  /app/static
# 固定 uid/gid = 10001:运行用户身份稳定,部署脚本
# 才能把 bind mount 的 ./data ./logs chown 到这个确定的 uid。
RUN groupadd --system --gid 10001 tinyaxum \
 && useradd --system --uid 10001 --gid 10001 \
      --no-create-home --shell /usr/sbin/nologin tinyaxum \
 && mkdir -p /app/data /app/logs \
 && chown -R tinyaxum:tinyaxum /app
USER tinyaxum
EXPOSE 3001
ENTRYPOINT ["/app/tinyaxum"]
