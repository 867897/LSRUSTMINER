#!/usr/bin/env bash

# LSRustMiner installer for Ubuntu/Debian.
# Source repository: https://github.com/867897/LSRUSTMINER

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

REPO_URL="https://github.com/867897/LSRUSTMINER.git"
REPO_BRANCH="${REPO_BRANCH:-}"
INSTALL_DIR="/opt/lsrustminer"
SERVICE_NAME="lsrustminer"
BINARY_NAME="LSRustMiner"
LINUX_BINARY_PATH="Linux/${BINARY_NAME}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR=""

check_root() {
    if [ "${EUID}" -ne 0 ]; then
        echo -e "${RED}错误: 请使用 root 权限运行此脚本${NC}"
        echo "用法: sudo bash $0"
        exit 1
    fi
}

ensure_tools() {
    if ! command -v git >/dev/null 2>&1; then
        echo -e "${YELLOW}正在安装 git...${NC}"
        apt-get update
        apt-get install -y git
    fi
}

print_logo() {
    echo -e "${BLUE}"
    echo "=========================================="
    echo "        LSRustMiner 安装管理器"
    echo "        GitHub: 867897/LSRUSTMINER"
    echo "=========================================="
    echo -e "${NC}"
}

check_status() {
    if systemctl is-active --quiet "${SERVICE_NAME}" 2>/dev/null; then
        echo -e "${GREEN}运行中${NC}"
        return 0
    fi
    echo -e "${RED}未运行${NC}"
    return 1
}

fetch_package() {
    if [ -f "${SCRIPT_DIR}/${BINARY_NAME}" ]; then
        echo "${SCRIPT_DIR}/${BINARY_NAME}"
        return 0
    fi

    if [ -f "${SCRIPT_DIR}/${LINUX_BINARY_PATH}" ]; then
        echo "${SCRIPT_DIR}/${LINUX_BINARY_PATH}"
        return 0
    fi

    ensure_tools
    TMP_DIR="$(mktemp -d)"
    echo -e "${YELLOW}正在从 GitHub 拉取安装包...${NC}" >&2
    if [ -n "${REPO_BRANCH}" ]; then
        git clone --depth 1 --branch "${REPO_BRANCH}" "${REPO_URL}" "${TMP_DIR}/repo" >&2
    else
        git clone --depth 1 "${REPO_URL}" "${TMP_DIR}/repo" >&2
    fi
    echo "${TMP_DIR}/repo/${LINUX_BINARY_PATH}"
}

cleanup_tmp() {
    if [ -n "${TMP_DIR}" ] && [ -d "${TMP_DIR}" ]; then
        rm -rf "${TMP_DIR}"
    fi
}

create_service() {
    cat > "/etc/systemd/system/${SERVICE_NAME}.service" << EOF
[Unit]
Description=LSRustMiner Mining Proxy
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=${INSTALL_DIR}
ExecStart=${INSTALL_DIR}/${BINARY_NAME}
Restart=always
RestartSec=5
LimitNOFILE=1048576
LimitNPROC=1048576
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
    echo -e "${GREEN}systemd 服务文件已创建${NC}"
}

install_program() {
    echo -e "${YELLOW}正在安装 LSRustMiner...${NC}"

    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        echo -e "${RED}程序已安装。如需覆盖安装，请选择“重新安装”。${NC}"
        return 1
    fi

    local package_dir
    package_dir="$(fetch_package)"

    if [ ! -f "${package_dir}" ]; then
        echo -e "${RED}错误: 未找到服务端二进制文件 ${LINUX_BINARY_PATH}${NC}"
        echo -e "${YELLOW}请确认 GitHub 仓库中存在 Linux/${BINARY_NAME}。${NC}"
        return 1
    fi

    mkdir -p "${INSTALL_DIR}/data"

    cp "${package_dir}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    local package_root
    package_root="$(dirname "$(dirname "${package_dir}")")"
    if [ -d "${package_root}/static" ]; then
        rm -rf "${INSTALL_DIR}/static"
        cp -r "${package_root}/static" "${INSTALL_DIR}/static"
    fi

    create_service
    systemctl daemon-reload
    systemctl enable "${SERVICE_NAME}"
    systemctl start "${SERVICE_NAME}"

    echo -e "${GREEN}安装完成！${NC}"
    echo "安装目录: ${INSTALL_DIR}"
    echo "配置目录: ${INSTALL_DIR}/data"
    echo -e "服务状态: $(check_status)"
}

reinstall_program() {
    echo -e "${YELLOW}警告: 重新安装会删除程序、配置和数据。${NC}"
    read -r -p "确认重新安装? (y/N): " confirm
    if [ "${confirm}" != "y" ] && [ "${confirm}" != "Y" ]; then
        echo "已取消"
        return 0
    fi

    uninstall_program_silent
    install_program
}

unlock_limits() {
    echo -e "${YELLOW}正在解除 Linux 连接数限制...${NC}"

    cp /etc/security/limits.conf /etc/security/limits.conf.bak 2>/dev/null || true
    cp /etc/sysctl.conf /etc/sysctl.conf.bak 2>/dev/null || true

    grep -q "# LSRustMiner Limits" /etc/security/limits.conf || cat >> /etc/security/limits.conf << EOF

# LSRustMiner Limits
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 1048576
* hard nproc 1048576
root soft nofile 1048576
root hard nofile 1048576
root soft nproc 1048576
root hard nproc 1048576
EOF

    grep -q "# LSRustMiner Network Tuning" /etc/sysctl.conf || cat >> /etc/sysctl.conf << EOF

# LSRustMiner Network Tuning
fs.file-max = 1048576
fs.nr_open = 1048576
net.ipv4.tcp_max_syn_backlog = 65535
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 3
net.ipv4.tcp_keepalive_intvl = 30
net.ipv4.ip_local_port_range = 1024 65535
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
EOF

    sysctl -p
    ulimit -n 1048576 2>/dev/null || true
    ulimit -u 1048576 2>/dev/null || true

    echo -e "${GREEN}连接数限制已处理。部分设置可能需要重启系统后生效。${NC}"
    echo "当前文件描述符限制: $(ulimit -n)"
}

stop_program() {
    echo -e "${YELLOW}正在停止 LSRustMiner...${NC}"
    if systemctl is-active --quiet "${SERVICE_NAME}" 2>/dev/null; then
        systemctl stop "${SERVICE_NAME}"
        echo -e "${GREEN}程序已停止${NC}"
    else
        echo -e "${YELLOW}程序未运行${NC}"
    fi
}

start_program() {
    echo -e "${YELLOW}正在启动 LSRustMiner...${NC}"
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        echo -e "${RED}程序未安装${NC}"
        return 1
    fi
    systemctl start "${SERVICE_NAME}"
    sleep 1
    echo -e "服务状态: $(check_status)"
}

restart_program() {
    echo -e "${YELLOW}正在重启 LSRustMiner...${NC}"
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        echo -e "${RED}程序未安装${NC}"
        return 1
    fi
    systemctl restart "${SERVICE_NAME}"
    sleep 1
    echo -e "服务状态: $(check_status)"
}

uninstall_program_silent() {
    systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
    systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
    rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
    systemctl daemon-reload
    rm -rf "${INSTALL_DIR}"
}

uninstall_program() {
    echo -e "${YELLOW}警告: 卸载会删除程序、配置和数据。${NC}"
    read -r -p "确认卸载? (y/N): " confirm
    if [ "${confirm}" != "y" ] && [ "${confirm}" != "Y" ]; then
        echo "已取消"
        return 0
    fi

    uninstall_program_silent
    echo -e "${GREEN}卸载完成！${NC}"
}

view_logs() {
    echo -e "${YELLOW}最近 50 条日志:${NC}"
    journalctl -u "${SERVICE_NAME}" -n 50 --no-pager
}

show_menu() {
    echo ""
    echo -e "${BLUE}==========================================${NC}"
    echo -e "当前状态: $(check_status)"
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        echo -e "安装目录: ${GREEN}${INSTALL_DIR}${NC}"
    fi
    echo -e "${BLUE}==========================================${NC}"
    echo ""
    echo "  1. 安装程序"
    echo "  2. 重新安装程序 (删除原有数据)"
    echo "  3. 解除 Linux 连接数限制"
    echo "  4. 启动程序"
    echo "  5. 停止程序"
    echo "  6. 重启程序"
    echo "  7. 查看日志"
    echo "  8. 卸载程序"
    echo "  0. 退出"
    echo ""
}

main() {
    trap cleanup_tmp EXIT
    check_root

    while true; do
        clear
        print_logo
        show_menu

        read -r -p "请选择操作 [0-8]: " choice
        echo ""

        case "${choice}" in
            1) install_program ;;
            2) reinstall_program ;;
            3) unlock_limits ;;
            4) start_program ;;
            5) stop_program ;;
            6) restart_program ;;
            7) view_logs ;;
            8) uninstall_program ;;
            0)
                echo -e "${GREEN}再见！${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}无效选项，请重新选择${NC}"
                ;;
        esac

        echo ""
        read -r -p "按回车键继续..."
    done
}

main
