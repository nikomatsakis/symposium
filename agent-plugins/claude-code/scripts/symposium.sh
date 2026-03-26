#!/usr/bin/env bash
# Symposium bootstrap and forwarding script.
# Finds an existing symposium binary or downloads one, then forwards all arguments.

set -euo pipefail

SYMPOSIUM_DIR="${HOME}/.symposium"
REPO="symposium-dev/symposium"

find_binary() {
    # 1. Check ~/.cargo/bin (cargo install / cargo binstall)
    if [ -x "${HOME}/.cargo/bin/symposium" ]; then
        echo "${HOME}/.cargo/bin/symposium"
        return 0
    fi

    # 2. Check ~/.symposium (our install location)
    if [ -x "${SYMPOSIUM_DIR}/symposium" ]; then
        echo "${SYMPOSIUM_DIR}/symposium"
        return 0
    fi

    # 3. Check PATH
    if command -v symposium >/dev/null 2>&1; then
        command -v symposium
        return 0
    fi

    return 1
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}" in
        Darwin)
            case "${arch}" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                *) echo >&2 "Unsupported macOS architecture: ${arch}"; exit 1 ;;
            esac
            ;;
        Linux)
            case "${arch}" in
                x86_64)  echo "x86_64-unknown-linux-musl" ;;
                aarch64) echo "aarch64-unknown-linux-musl" ;;
                *) echo >&2 "Unsupported Linux architecture: ${arch}"; exit 1 ;;
            esac
            ;;
        *)
            echo >&2 "Unsupported OS: ${os}"
            exit 1
            ;;
    esac
}

download_binary() {
    local target url
    target="$(detect_target)"
    url="https://github.com/${REPO}/releases/latest/download/symposium-${target}.tar.gz"

    echo >&2 "Downloading symposium for ${target}..."

    DOWNLOAD_TMPDIR="$(mktemp -d)"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "${url}" -o "${DOWNLOAD_TMPDIR}/symposium.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "${url}" -O "${DOWNLOAD_TMPDIR}/symposium.tar.gz"
    else
        echo >&2 "Error: neither curl nor wget found"
        exit 1
    fi

    mkdir -p "${SYMPOSIUM_DIR}"
    tar -xzf "${DOWNLOAD_TMPDIR}/symposium.tar.gz" -C "${SYMPOSIUM_DIR}"
    chmod +x "${SYMPOSIUM_DIR}/symposium"
    rm -rf "${DOWNLOAD_TMPDIR}"

    echo >&2 "Installed symposium to ${SYMPOSIUM_DIR}/symposium"
}

# Find or download the binary
BINARY="$(find_binary)" || {
    download_binary
    BINARY="${SYMPOSIUM_DIR}/symposium"
}

# Forward all arguments
exec "${BINARY}" "$@"
