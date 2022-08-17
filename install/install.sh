#!/usr/bin/env sh

set -e

if ! command -v unzip >/dev/null; then
	echo "Error: unzip is required to install chost." 1>&2
	exit 1
fi

if [ "$OS" = "Windows_NT" ]; then
	echo "Error: chost currently only works in macOS" 1>&2
	exit 1
else
	case $(uname -sm) in
	"Darwin x86_64") target="x86_64-apple-darwin"
	;;
	"Darwin arm64") target="aarch64-apple-darwin"
	;;
	*)
	echo "Error: chost currently only works in macOS" 1>&2
	exit 1
	;;
	esac
fi

chost_uri="https://github.com/TrickyPi/chost/releases/latest/download/chost-${target}.zip"

chost_install="$HOME/.chost"
bin_dir="$chost_install/bin"
exe="$bin_dir/chost"

if [ ! -d "$bin_dir" ]; then
    mkdir -p "$bin_dir"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$chost_uri"
tar xfC "$exe.zip" "$bin_dir"
chmod +x "$exe"
rm "$exe.zip"

echo "chost was installed successfully to $exe"

if command -v chost >/dev/null; then
	echo "Run 'chost --help' to get started"
else
	case $SHELL in
	/bin/zsh) shell_profile=".zshrc" ;;
	*) shell_profile=".bashrc" ;;
	esac
	echo "Manually add the following stuff to your \$HOME/$shell_profile (or similar)"
	echo "  export PATH=\"$bin_dir:\$PATH\""
fi