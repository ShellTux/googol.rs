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
			vm_bin="$(nixos-rebuild build-vm --flake .#"$profile" 2>&1 | grep --only-matching '/nix/store/\S\+')"
			QEMU_KERNEL_PARAMS=console=ttyS0 $vm_bin -nographic; reset
		)
		;;
	run-graphic)
		(
			set -x;
			vm_bin="$(nixos-rebuild build-vm --flake .#"$profile" 2>&1 | grep --only-matching '/nix/store/\S\+')"
			$vm_bin
		)
		;;
	*)
		usage
		;;
esac
