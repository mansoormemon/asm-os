qemu-system-x86_64 \
  -drive file=target/x86_64-asm-os-kernel/release/bootimage-asm-os.bin,format=raw \
  -m 1G \
  -smp cpus=4,cores=4,threads=1,sockets=1 \
