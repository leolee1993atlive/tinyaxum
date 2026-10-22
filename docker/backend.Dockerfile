# ---------- runtime ----------
# 注意:二进制在宿主机编译(build-docker.sh),动态链接宿主的 glibc。
# 运行镜像的 glibc 必须 >= 编译机,否则启动时报 `GLIBC_x.y not found` 崩溃重启。
# 故此处与编译机(Ubuntu 24.04 / glibc 2.39)对齐,使用 ubuntu:24.04。
# 换编译机时务必同步这里的基础镜像。
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3t64 libstdc++6 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY hdata-analysis-rs/target/release/hdata-analysis /app/hdata-analysis
COPY hdata-analysis-ui/build                          /app/web
# DuckDB 离线扩展(build-docker.sh 按 Cargo 的 duckdb 版本下载到 docker/extensions/)。
# 必须放在 exe 同目录的 extensions/ 下,程序才会走离线 LOAD,免去容器联网安装。
COPY docker/extensions                                /app/extensions
# 固定 uid/gid = 10001:运行用户身份稳定,部署脚本(load-and-start.sh)
# 才能把 bind mount 的 ./data ./logs chown 到这个确定的 uid。
# 改这里务必同步 load-and-start.sh 里的 chown 目标。
RUN groupadd --system --gid 10001 hdata \
 && useradd --system --uid 10001 --gid 10001 \
      --no-create-home --shell /usr/sbin/nologin hdata \
 && mkdir -p /app/data /app/logs \
 && chown -R hdata:hdata /app
USER hdata
EXPOSE 3000
ENTRYPOINT ["/app/hdata-analysis"]
