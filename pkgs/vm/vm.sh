#!/bin/sh
set -e

usage() {
	echo "$0 <subcommand> <vm-profile>"
	echo 'Subcommands:'
	echo '  - dry-build'
	echo '  - build'
	echo '  - run'
	echo '  - run-graphic'

	exit 1
}

[ "$#" -ne 2 ] && usage

profile="$2"
vm_bin=./result/bin/"run-$profile-vm"

case "$1" in
	dry-build)
		(
			set -x;
			nixos-rebuild dry-build --flake .#"$profile"
		)
		;;
	build)
		(
			set -x;
			nixos-rebuild build-vm --flake .#"$profile"
		)
		;;
	run)
		(
			set -x;
			nixos-rebuild build-vm --flake .#"$profile"
			QEMU_KERNEL_PARAMS=console=ttyS0 $vm_bin -nographic; reset
		)
		;;
	run-graphic)
		(
			set -x;
			nixos-rebuild build-vm --flake .#"$profile"
			$vm_bin
		)
		;;
	*)
		usage
		;;
esac
