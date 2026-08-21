NVME_IMG := nvme-1.img
NVME_SIZE := 50G
PACKAGES := $(shell cargo metadata --format-version 1 | jq -r '.workspace_members[]' | sed -E 's|.*/([^ # ]+)#.*|\1|')
CARGO := cargo
QEMU := sudo qemu-system-x86_64

# Paths for OVMF firmware, disk image, and ISO file
OVMF_PATH := /usr/share/edk2/ovmf/OVMF_CODE.fd
IMAGE_PATH := target/amentys-uefi.img
ISO_PATH := target/amentys.iso
TIMEOUT := 180
SUB_PROC := 4

ifeq ($(SUB_PROC), 0)
  SUB_PROC := 2
else
  SUB_PROC := $(SUB_PROC)
endif
# Calculate the number of jobs to run in parallel based on the number of CPU cores available
NUM_JOBS := $(shell echo $$(($(shell nproc) - $(SUB_PROC))))

ifneq ($(NUM_JOBS), 0)
  NUM_JOBS := $(NUM_JOBS)
else
  NUM_JOBS := 1
endif

# Configure QEMU for launching the raw disk image
QEMU_FLAGS := -machine q35 -bios $(OVMF_PATH) \
              -enable-kvm -m 2G \
              -vga virtio \
              -device ahci,id=ahci \
              -drive format=raw,file=$(IMAGE_PATH),if=none,id=bootdisk \
              -device ide-hd,bus=ahci.0,drive=bootdisk \
              -drive file=$(NVME_IMG),if=none,id=drv1,format=raw \
              -device nvme,drive=drv1,serial=nvme-1 \
              -serial stdio \
              -device isa-debug-exit,iobase=0xf4,iosize=0x04 --full-screen -d int,cpu_reset -no-shutdown -no-reboot
QEMU_FLAGS += -d int,guest_errors -D qemu.log

# Configure QEMU for launching the ISO file
QEMU_ISO_FLAGS := -machine q35 -bios $(OVMF_PATH) \
                  -enable-kvm -m 2G \
                  -vga virtio \
                  -drive format=raw,file=$(ISO_PATH),index=0,media=cdrom \
                  -drive file=$(NVME_IMG),if=none,id=drv1,format=raw \
                  -device nvme,drive=drv1,serial=nvme-1 \
                  -serial stdio \
                  -device isa-debug-exit,iobase=0xf4,iosize=0x04 --full-screen --no-reboot

# User Profile: Absolute prohibition of panicking or crashing wildly
CLIPPY_USER_FLAGS := -D warnings \
                     -D clippy::undocumented_unsafe_blocks \
                     -D clippy::missing_safety_doc \
                     -D clippy::ptr_as_ptr \
                     -D clippy::transmute_ptr_to_ptr \
                     -D clippy::inline_always \
                     -D clippy::panic \
                     -A clippy::unwrap_used \
                     -A clippy::expect_used \
                     -D clippy::cast_possible_truncation \
                     -D clippy::cast_possible_wrap \
                     -D clippy::std_instead_of_core \
                     -W clippy::pedantic \
                     -W clippy::nursery \
                     -A clippy::missing_errors_doc

# System Profile (Kernel): Allows the indispensable panic!() and expect() in Ring 0
CLIPPY_KERNEL_FLAGS := -D warnings \
                       -D clippy::undocumented_unsafe_blocks \
                       -D clippy::missing_safety_doc \
                       -D clippy::ptr_as_ptr \
                       -D clippy::transmute_ptr_to_ptr \
                       -D clippy::inline_always \
                       -D clippy::cast_possible_truncation \
                       -D clippy::cast_possible_wrap \
                       -D clippy::std_instead_of_core \
                       -W clippy::pedantic \
                       -W clippy::nursery \
                       -A clippy::missing_errors_doc \
                       -A clippy::panic \
                       -A clippy::expect_used

.PHONY: help ci clean run bootimage iso run-iso image up documentation doc

# Terminal display color codes
C_RESET := \033[0m
C_OK := \033[32;1m
C_KO := \033[31;1m
C_INFO := \033[1;37;1m

help: ## Print this help list with all available commands
	@$(call center_text,Amentys OS)
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[1;32m%-15s\033[1;37m %s\033[0m\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
ci_all:
	@$(call test_packages)
# Nouvelle target avec timeout de 30 secondes
ci: clean ## Run all tests (cargo test, clippy, fmt, machete) on the packages with 30s timeout
	@$(call clean_logs)
	@$(call disable_config)
	@Termwidth=$$(tput cols 2>/dev/null || echo 80); \
	$(call disable_cursor)
	@timeout --preserve-status $(TIMEOUT) $(MAKE) ci_all || \
	{ \
		echo -e "\n\t\t\t$(C_KO)    Tests timed out after $(TIMEOUT) seconds!$(C_RESET)\n"; \
		exit 1; \
	}
	@$(call enable_cursor)
	@$(call enable_config)
hub: ## Start a local Mercurial server to serve the repository
	@hg serve
image: ci ## Compile the kernel and init, then generate the raw disk image (.img)
	@$(call disable_cursor)
	@$(call make_image)
	@$(call enable_cursor)

iso: ci ## Compile the kernel and init, then generate the bootable image (.iso)
	@$(call disable_cursor)
	@$(call make_iso)
	@$(call enable_cursor)

clean: ## Clean the project (remove the target folder, logs, and reformat the code)
	@clear
	@cargo fmt --all
	@rm -rf ci/stdout ci/stderr target/ ISO/
	@rm -f $(IMAGE_PATH) $(ISO_PATH)
launch:
	@$(call make_image)
	@$(call run)
up: image ## Compile the disk image and launch it in QEMU with KVM
	@$(call center_text, AMENTYS (IMG))
	@if [ ! -f $(NVME_IMG) ]; then qemu-img create -f raw $(NVME_IMG) $(NVME_SIZE) --quiet; fi
	@$(call run)

run-iso: iso ## Compile the ISO and launch it in QEMU with KVM
	@$(call center_text, AMENTYS (ISO))
	@if [ ! -f $(NVME_IMG) ]; then qemu-img create -f raw $(NVME_IMG) $(NVME_SIZE) --quiet; fi
	@$(call run_iso)

doc: documentation ## Generate the Rust documentation for the project and open it in the browser
	@$(call run-doc)

documentation: ## Generate the Rust documentation for the project
	@$(call disable_cursor)
	@$(call make_doc)
	@$(call enable_cursor)

debug: ## Display detected packages and CPU cores used for compilation
	echo -e "Detected packages  : $(C_OK)$(PACKAGES)$(C_RESET)"
	echo -e "Detected cpu cores : $(C_OK)$(NUM_JOBS)$(C_RESET) cpus used for compilation"

# Macro to clean logs
define clean_logs
	@rm -rf ci/stdout ci/stderr
endef
# Macro to run QEMU with the specified flags
define run
	@$(QEMU) $(QEMU_FLAGS)
endef

# Macro to run QEMU with the ISO flags
define run_iso
	@$(QEMU) $(QEMU_ISO_FLAGS)
endef

# Macro to compile the UEFI image with Limine
define make_image
	@$(call center_text, Building kernel (re))
	@RUSTFLAGS="-Zunstable-options" RUST_TARGET_PATH=$(shell pwd) $(CARGO) build --jobs $(NUM_JOBS) --bin re --release --target x86_64-amentys
	@$(call center_text, Building init (maat))
	# Using the standard target without a custom linker.ld
	@RUSTFLAGS="-Zunstable-options" $(CARGO) build --jobs $(NUM_JOBS) --bin maat --release --target x86_64-unknown-none
	@$(call center_text, Creating UEFI Boot Image with Limine)
	@mkdir -p target/
	@dd if=/dev/zero of=$(IMAGE_PATH) bs=1M count=64 status=none
	@mformat -i $(IMAGE_PATH) -F ::
	@mmd -i $(IMAGE_PATH) ::/EFI
	@mmd -i $(IMAGE_PATH) ::/EFI/BOOT
	@mmd -i $(IMAGE_PATH) ::/boot
	@if [ ! -f target/BOOTX64.EFI ]; then cp BOOTX64.EFI target/BOOTX64.EFI; fi
	@mcopy -i $(IMAGE_PATH) target/BOOTX64.EFI ::/EFI/BOOT/BOOTX64.EFI
	@mcopy -i $(IMAGE_PATH) limine.conf ::/limine.conf
	@if [ -f "wallpaper.jpg" ]; then mcopy -i $(IMAGE_PATH) wallpaper.jpg ::/wallpaper.jpg; fi
	@mcopy -i $(IMAGE_PATH) target/x86_64-amentys/release/re ::/re
	@mcopy -i $(IMAGE_PATH) target/x86_64-unknown-none/release/maat ::/maat
	@$(call center_text, Image $(IMAGE_PATH) ready!)
endef

# Macro to compile the bootable ISO with Limine
define make_iso
	@$(call center_text, Building kernel (re))
	@RUSTFLAGS="-Zunstable-options" RUST_TARGET_PATH=$(shell pwd) $(CARGO) build --jobs $(NUM_JOBS) --bin re --release --target x86_64-amentys
	@$(call center_text, Building init (maat))
	# Using the standard target without a custom linker.ld
	@RUSTFLAGS="-Zunstable-options" $(CARGO) build --jobs $(NUM_JOBS) --bin maat --release --target x86_64-unknown-none
	@$(call center_text, Creating Bootable ISO with Limine)
	@mkdir -p ISO/boot ISO/EFI/BOOT
	@if [ -f BOOTX64.EFI ]; then cp BOOTX64.EFI ISO/EFI/BOOT/BOOTX64.EFI; fi
	@cp limine.conf ISO/limine.conf
	@cp limine.conf ISO/boot/limine.conf
	@if [ -f "wallpaper.png" ]; then cp wallpaper.png ISO/wallpaper.png; fi
	
	@cp target/x86_64-amentys/release/re ISO/boot/re
	@cp target/x86_64-unknown-none/release/maat ISO/boot/maat
	@if [ -f limine-uefi-cd.bin ]; then cp limine-uefi-cd.bin ISO/limine-uefi-cd.bin; fi
	
	@xorriso -as mkisofs -R -J \
		-no-emul-boot \
		-boot-load-size 4 \
		-boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image \
		ISO -o $(ISO_PATH)
		
	@limine bios-install $(ISO_PATH) 2>/dev/null || true
	@$(call center_text, ISO $(ISO_PATH) ready!)
endef

# Dynamically displays centered text according to the terminal size
define center_text
	termwidth=$$(tput cols 2>/dev/null || echo 80); \
	text="$(1)"; \
	pad=$$(( (termwidth - $${#text}) / 2 )); \
	printf "\n$(C_INFO)%*s$(C_RESET)\n\n" $$((pad + $${#text})) "$$text"
endef

# Macro to generate and open the Rust documentation
define run-doc
	@RUST_TARGET_PATH=$(shell pwd) $(CARGO) doc --no-deps --document-private-items --open
endef

# Macro to generate the Rust documentation for the project
define make_doc
	@$(call center_text, Generating documentation)
	@RUST_TARGET_PATH=$(shell pwd) $(CARGO) doc --no-deps --document-private-items --jobs $(NUM_JOBS) --release
	@$(call center_text, Documentation generated in target/doc/)
endef

# Comprehensive test suite for all workspace packages
define test_packages
	@$(call disable_config)
	status=0; \
	termwidth=$$(tput cols 2>/dev/null || echo 80); \
	row_width=97; \
	pad=$$(( (termwidth - row_width) / 2 )); \
	pad=$$(($$pad > 0 ? $$pad : 0)); \
	pad_str=$$(printf '%*s' "$$pad" ""); \
	printf "\n$$pad_str$(C_INFO)%-15s  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %s$(C_RESET)\n" "PACKAGE" "START" "END" "TEST" "CHECK" "CLIPPY" "FMT" "MACHETE" "  STATUS  "; \
	for pkg in $(PACKAGES); do \
		mkdir -p ci/stdout/$$pkg ci/stderr/$$pkg; \
		test=0; check=0; clippy=0; fmt=0; machete=0; \
		s="success"; f="failure"; \
		bd=$$(date +'%H:%M:%S'); \
		printf "\r$$pad_str$(C_INFO)%-15s$(C_RESET)  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %s" "$$pkg" "$$bd" "unknown " "unknown " "unknown " "unknown " "unknown " "unknown " " unknown  "; \
		if [ "$$pkg" = "re" ]; then flags="$(CLIPPY_KERNEL_FLAGS)"; else flags="$(CLIPPY_USER_FLAGS)"; fi; \
		if [ "$$pkg" != "re" ] && [ "$$pkg" != "maat" ] && [ "$$pkg" != "ji" ] && [ "$$pkg" != "zuu" ] && [ "$$pkg" != "hu" ]; then \
			$(CARGO) test -p $$pkg --lib --release --jobs=$$(nproc) > ci/stdout/$$pkg/test.log 2> ci/stderr/$$pkg/test.log || test=1; \
		fi; \
		$(CARGO) check -p $$pkg --lib --release --jobs=$$(nproc) > ci/stdout/$$pkg/check.log 2> ci/stderr/$$pkg/check.log || check=1; \
		$(CARGO) clippy -p $$pkg --release --jobs=$$(nproc) -- $$flags > ci/stdout/$$pkg/clippy.log 2> ci/stderr/$$pkg/clippy.log || clippy=1; \
		$(CARGO) fmt -p $$pkg -- --check > ci/stdout/$$pkg/fmt.log 2> ci/stderr/$$pkg/fmt.log || fmt=1; \
		$(CARGO) machete $$pkg > ci/stdout/$$pkg/machete.log 2> ci/stderr/$$pkg/machete.log || machete=1; \
		if [ "$$test" -eq 0 ] && [ "$$check" -eq 0 ] && [ "$$clippy" -eq 0 ] && [ "$$fmt" -eq 0 ] && [ "$$machete" -eq 0 ]; then \
			printf "\r$$pad_str$(C_INFO)%-15s$(C_RESET)  %-8s  %-8s  $(C_OK)%-8s$(C_RESET)  $(C_OK)%-8s$(C_RESET)  $(C_OK)%-8s$(C_RESET)  $(C_OK)%-8s$(C_RESET)  $(C_OK)%-8s$(C_RESET)  $(C_OK)%s$(C_RESET)\n" "$$pkg" "$$bd" "$$(date +'%H:%M:%S')" "$$s" "$$s" "$$s" "$$s" "$$s" "    OK    "; \
		else \
			t_c=$$([ "$$test" -eq 0 ] && echo "$(C_OK)" || echo "$(C_KO)"); \
			c_c=$$([ "$$check" -eq 0 ] && echo "$(C_OK)" || echo "$(C_KO)"); \
			cl_c=$$([ "$$clippy" -eq 0 ] && echo "$(C_OK)" || echo "$(C_KO)"); \
			f_c=$$([ "$$fmt" -eq 0 ] && echo "$(C_OK)" || echo "$(C_KO)"); \
			m_c=$$([ "$$machete" -eq 0 ] && echo "$(C_OK)" || echo "$(C_KO)"); \
			t_s=$$([ "$$test" -eq 0 ] && echo "$$s" || echo "$$f"); \
			c_s=$$([ "$$check" -eq 0 ] && echo "$$s" || echo "$$f"); \
			cl_s=$$([ "$$clippy" -eq 0 ] && echo "$$s" || echo "$$f"); \
			f_s=$$([ "$$fmt" -eq 0 ] && echo "$$s" || echo "$$f"); \
			m_s=$$([ "$$machete" -eq 0 ] && echo "$$s" || echo "$$f"); \
			printf "\r$$pad_str$(C_INFO)%-15s$(C_RESET)  %-8s  %-8s  $${t_c}%-8s$(C_RESET)  $${c_c}%-8s$(C_RESET)  $${cl_c}%-8s$(C_RESET)  $${f_c}%-8s$(C_RESET)  $${m_c}%-8s$(C_RESET)  $(C_KO)%s$(C_RESET)\n" "$$pkg" "$$bd" "$$(date +'%H:%M:%S')" "$$t_s" "$$c_s" "$$cl_s" "$$f_s" "$$m_s" "    KO    "; \
			status=1; \
		fi; \
	done; \
	printf "\n$$pad_str$(C_INFO)%-15s$(C_RESET)\n\n" "completed"; \
	exit $$status
endef

# Cargo configuration management to isolate the environment
define disable_config
	mv .cargo/config.toml .cargo/config.toml.tmp 2>/dev/null || true;
endef

# Cargo configuration management to restore the environment
define enable_config
	mv .cargo/config.toml.tmp .cargo/config.toml 2>/dev/null || true;
endef

# Cargo configuration management to restore the environment (silent)
define restore_config
	mv .cargo/config.toml.tmp .cargo/config.toml 2>/dev/null || true;
endef

# Terminal cursor management
define disable_cursor
	tput civis 2>/dev/null || true;
endef
# Terminal cursor management
define enable_cursor
	tput cnorm 2>/dev/null || true;
endef
