#!/usr/bin/env bash

# If this is not your path, kindly change it
export CROSS_CC="${CROSS_CC:-$HOME/opt/cross/bin/i686-elf-gcc}"
export ARCH=x86/x32
export QEMU_SYSTEM=i386

# Colors
if command -v tput &> /dev/null; then
    ncolors=$(tput colors)
    if [ -n "$ncolors" ] && [ $ncolors -ge 8 ]; then
		bold="$(tput bold)"
        underline="$(tput smul)"
        standout="$(tput smso)"
        normal="$(tput sgr0)"
        black="$(tput setaf 0)"
        red="$(tput setaf 1)"
        green="$(tput setaf 2)"
        yellow="$(tput setaf 3)"
        blue="$(tput setaf 4)"
        magenta="$(tput setaf 5)"
        cyan="$(tput setaf 6)"
        white="$(tput setaf 7)"
	fi
fi

# Utils
build() {
	if ! cargo build; then
		return -1
	fi

	KERNEL="target/target/debug/satan"
	GRUB_CFG="src/arch/$ARCH/grub"
	if [ "$KERNEL" -nt "bin/os.iso" ] || [ "$GRUB_CFG" -nt "bin/os.iso" ]; then
		echo "${green}Building the system...${normal}"
		rm -rf bin/iso bin/os.iso
		mkdir -p bin/iso/boot
		cp -r "$GRUB_CFG" bin/iso/boot
		cp "$KERNEL" bin/iso/boot/kernel.bin
		grub-mkrescue -o bin/os.iso bin/iso
		echo "${green}Done!${normal}"
	fi
}

run() {
	qemu-system-$QEMU_SYSTEM -d guest_errors -no-reboot -cdrom bin/os.iso
}

debug() {
	qemu-system-$QEMU_SYSTEM -d guest_errors -no-reboot -cdrom bin/os.iso -s -S &
	rust-gdb target/target/debug/satan -x gdbinit
}

clean() {
	rm -rf bin target
}

print() {
	echo "ARCH: $ARCH"
	echo "CROSS_CC: $CROSS_CC"
	echo "QEMU System: $QEMU_SYSTEM"
}

help() {
	cat <<-EOF
	SATAN Build system
	Usage: ./build.sh [--arch x86] [--toolchain i686-elf] [--quemu-system x86_64] [command]
	When ran without command, REPL mode will be entered
	Commands:
	build - build kernel and OS
	run - build and run the OS
	debug - build and run the OS, drop into gdb
	print - print current parameters
	clean - remove all build artifacts
	help - show this message
	quit/exit/q - exit REPL
	EOF
}

command() {
	case $1 in
		build) build;;
		run) build && run;;
		debug) build && debug;;
		print) print;;
		clean)  clean;;
		help)  help;;
		quit|exit|q)  exit 0;;
		*) echo -e "Unknown command $1.\nUse help to see the available options";;
	esac
}

repl() {
	cat <<-"EOF"
	 .oooooo..o       .o.       ooooooooooooo       .o.       ooooo      ooo
	d8P'    `Y8      .888.      8'   888   `8      .888.      `888b.     `8'
	Y88bo.          .8"888.          888          .8"888.      8 `88b.    8
	 `"Y8888o.     .8' `888.         888         .8' `888.     8   `88b.  8
	     `"Y88b   .88ooo8888.        888        .88ooo8888.    8     `88b.8
	oo     .d8P  .8'     `888.       888       .8'     `888.   8       `888
	8""88888P'  o88o     o8888o     o888o     o88o     o8888o o8o        `8
	
	Type help for list of availble commands
	EOF
 	while true; do
		if command -v zsh &> /dev/null; then
			REPLY=$(zsh -c 'export REPLY="" && vared -p "> " REPLY && echo $REPLY')
		else
			read -e -r -p "> "
		fi
 		if [[ $REPLY == \:* ]]; then
 			bash -c "${REPLY#\:}"
 		else
	 		command $REPLY
 		fi
 	done
}

# Main
POSITIONAL_ARGS=()

while [[ $# -gt 0 ]]; do
	case $1 in
		--arch)
			export ARCH="$2"
			shift
			shift
		;;
		--cross-cc)
			export CROSS_CC="$2"
			shift
			shift
		;;
		--qemu-system)
			export QEMU_SYSTEM="$2"
			shift
			shift
		;;
		-h|--help)
			help
			exit 0
		;;
		-*|--*)
			echo "Unknown option $1"
			echo "Use help to see the available options"
			exit 1
		;;
    *)
      POSITIONAL_ARGS+=("$1")
      shift
      ;;
  esac
done

set -- "${POSITIONAL_ARGS[@]}"
if [ $# -gt 0 ]; then
	command "$@"
else
	repl
fi
