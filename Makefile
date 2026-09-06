SHELL := $(shell which bash)
NVME_IMG := nvme-1.img
NVME_SIZE := 50G
CARGO := cargo +nightly
QEMU := qemu-system-x86_64

# Paths for OVMF firmware, disk image, and ISO file
OVMF_CODE := $(firstword $(wildcard \
  /usr/share/OVMF/OVMF_CODE_4M.fd \
  /usr/share/OVMF/OVMF_CODE.fd \
  /usr/share/edk2/ovmf/OVMF_CODE.fd \
  /usr/share/edk2/x64/OVMF_CODE.4m.fd \
  /usr/share/edk2-ovmf/x64/OVMF_CODE.fd))

OVMF_VARS := $(firstword $(wildcard \
  /usr/share/OVMF/OVMF_VARS_4M.fd \
  /usr/share/OVMF/OVMF_VARS.fd \
  /usr/share/edk2/ovmf/OVMF_VARS.fd \
  /usr/share/edk2/x64/OVMF_VARS.4m.fd \
  /usr/share/edk2-ovmf/x64/OVMF_VARS.fd))

QEMU_SMP := 2
IMAGE_PATH := amentys-uefi.img
ISO_PATH := amentys.iso
TIMEOUT := 300
SUB_PROC := 2

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
QEMU := qemu-system-x86_64
QEMU_SMP := 2

QEMU_FLAGS := \
  -machine q35,accel=kvm \
  -cpu host \
  -smp $(QEMU_SMP) \
  -m 2G \
  -display gtk \
  -serial mon:stdio \
  -drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
  -drive if=pflash,format=raw,file=$(OVMF_VARS) \
  -drive if=none,format=raw,file=$(IMAGE_PATH),id=bootdisk \
  -device virtio-blk-pci,drive=bootdisk,bootindex=1 \
  -drive if=none,format=raw,file=$(NVME_IMG),id=drv1 \
  -device nvme,drive=drv1,serial=nvme-1 \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -no-reboot -no-shutdown

# Configure QEMU for launching the ISO file
QEMU_ISO_FLAGS := \
  -machine q35,accel=kvm \
  -cpu host \
  -smp $(QEMU_SMP) \
  -m 2G \
  -display gtk \
  -serial mon:stdio \
  -drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
  -drive if=pflash,format=raw,file=$(OVMF_VARS) \
  -drive if=none,format=raw,file=$(ISO_PATH),id=cdrom,media=cdrom \
  -device ide-cd,drive=cdrom,bootindex=1 \
  -drive if=none,format=raw,file=$(NVME_IMG),id=drv1 \
  -device nvme,drive=drv1,serial=nvme-1 \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -no-reboot -no-shutdown
# User Profile: Absolute prohibition of panicking or crashing wildly
CLIPPY_USER_FLAGS := -D warnings \
                     -D clippy::undocumented_unsafe_blocks \
                     -D clippy::missing_safety_doc \
                     -D clippy::ptr_as_ptr \
                     -D clippy::transmute_ptr_to_ptr \
                     -D clippy::inline_always \
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
                       -A clippy::expect_used

.PHONY: help ci clean run bootimage iso run-iso image up documentation doc demo

# Terminal display color codes
C_RESET := $$(printf '\033[0m')
C_GREEN := $$(printf '\033[32m')
C_MAGENTA := $$(printf '\033[35m')
C_RED := $$(printf '\033[31m')
C_CYAN := $$(printf '\033[36m')
C_BLUE := $$(printf '\033[34m')
C_YELLOW := $$(printf '\033[33m')
C_WHITE := $$(printf '\033[37m')
C_OK := $(C_GREEN)
C_KO := $(C_RED)
C_INFO := $(C_CYAN)

help: ## Print this help list with all available commands
	@$(call center_text,Amentys OS)
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""

demo:
	@python3 -m http.server -d demo/
ci: clean ## Run all tests (cargo test, clippy, fmt, machete) on the packages
	@$(call test_packages)
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
	@rm -rf ci/stdout ci/stderr target/ ISO/
	@rm -f $(IMAGE_PATH) $(ISO_PATH) nvme-1.img qemu.log
launch:
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
	@echo -e "Detected packages  : $(C_OK)$(PACKAGES)$(C_RESET)"
	@echo -e "Detected cpu cores : $(C_OK)$(NUM_JOBS)$(C_RESET) cpus used for compilation"
	@echo "$(QEMU) $(QEMU_FLAGS)"

# Macro to clean logs
define clean_logs
	@rm -rf ci/stdout ci/stderr
endef
# Macro to run QEMU with the specified flags
define run
	@sudo $(QEMU) $(QEMU_FLAGS)
endef

# Macro to run QEMU with the ISO flags
define run_iso
	@sudo $(QEMU) $(QEMU_ISO_FLAGS)
endef

# Macro to compile the UEFI image with Limine
define make_image
	@$(call center_text, Building kernel (re))
	RUSTFLAGS="-C relocation-model=static -C link-arg=-no-pie" $(CARGO) build --jobs $(NUM_JOBS) --target x86_64-unknown-none -Zbuild-std=core,alloc,compiler_builtins --bin re --release
	@$(call center_text, Building init (maat))
	# Using the standard target without a custom linker.ld
	RUSTFLAGS="-C relocation-model=static -C link-arg=-no-pie" $(CARGO) build --jobs $(NUM_JOBS) --target x86_64-unknown-none -Zbuild-std=core,alloc,compiler_builtins --bin maat --release
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
	@if [ -f "assets/wallpaper.jpg" ]; then mcopy -i $(IMAGE_PATH) assets/wallpaper.jpg ::/boot/wallpaper.jpg; fi
    @mcopy -i $(IMAGE_PATH) target/x86_64-unknown-none/release/re  ::/boot/re
    @mcopy -i $(IMAGE_PATH) target/x86_64-unknown-none/release/maat ::/boot/maat
	@$(call center_text, Image $(IMAGE_PATH) ready!)
endef

# Macro to compile the bootable ISO with Limine
define make_iso
@$(call center_text, Building kernel (re))
	RUSTFLAGS="-C relocation-model=static -C link-arg=-no-pie" $(CARGO) build --jobs $(NUM_JOBS) --target x86_64-unknown-none -Zbuild-std=core,alloc,compiler_builtins --bin re --release
	@$(call center_text, Building init (maat))
	# Using the standard target without a custom linker.ld
	RUSTFLAGS="-C relocation-model=static -C link-arg=-no-pie" $(CARGO) build --jobs $(NUM_JOBS) --target x86_64-unknown-none -Zbuild-std=core,alloc,compiler_builtins --bin maat --release
	@$(call center_text, Creating Bootable ISO with Limine)
	@mkdir -p ISO/boot ISO/EFI/BOOT
	@if [ -f BOOTX64.EFI ]; then cp BOOTX64.EFI ISO/EFI/BOOT/BOOTX64.EFI; fi
	@cp limine.conf ISO/limine.conf
	@cp limine.conf ISO/boot/limine.conf
	@if [ -f "assets/wallpaper.jpg" ]; then cp assets/wallpaper.jpg ISO/boot/wallpaper.jpg; fi
	@cp target/x86_64-unknown-none/release/re ISO/boot/re
	@cp target/x86_64-unknown-none/release/maat ISO/boot/maat
	@if [ -f limine-uefi-cd.bin ]; then cp limine-uefi-cd.bin ISO/limine-uefi-cd.bin; fi
	
	@xorriso -as mkisofs -R -J \
		-no-emul-boot \
		-V "AMENTYS" \
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
	@$(CARGO) doc --no-deps --document-private-items --open
endef

# Macro to generate the Rust documentation for the project
define make_doc
	@$(call center_text, Generating documentation)
	$(CARGO) doc --no-deps --document-private-items --jobs $(NUM_JOBS) --release
	@$(call center_text, Documentation generated in target/doc/)
endef

# Comprehensive test suite for all workspace packages
define test_packages
	@$(call center_text, Amentys Continuous Integration Testing)
	status=0; \
	termwidth=$$(tput cols 2>/dev/null || echo 80); \
	table_width=120; \
	pad=$$(( (termwidth - table_width) / 2 )); \
	pad=$$(($$pad > 0 ? $$pad : 0)); \
	pad_str=$$(printf '%*s' "$$pad" ""); \
	META=$$(cargo metadata --no-deps --format-version 1); \
	WORKSPACE_VERSION=$$(echo "$$META" | jq -r '.packages[0].version // "n/a"'); \
	WORKSPACE_EDITION=$$(echo "$$META" | jq -r '.packages[0].edition // "n/a"'); \
	mapfile -t pkgs < <(echo "$$META" | jq -r '.packages[].name'); \
	mapfile -t versions < <(echo "$$META" | jq -r '.packages[].version'); \
	mapfile -t editions < <(echo "$$META" | jq -r '.packages[].edition'); \
	mapfile -t descs < <(echo "$$META" | jq -r '.packages[].description // "N/A"'); \
	mkdir -p ci/stdout/amentys ci/stderr/amentys; \
	audit=0; deny=0; fail_steps=(); s="ok"; f="ko"; global_start=$$(date +'%H:%M:%S'); \
	if cargo audit --version >/dev/null 2>&1; then \
		$(CARGO) audit > ci/stdout/amentys/audit.log 2> ci/stderr/amentys/audit.log || audit=1; \
	else \
		echo "cargo audit not installed; failing CI" > ci/stdout/amentys/audit.log; \
		echo "cargo audit not installed; failing CI" > ci/stderr/amentys/audit.log; \
		audit=1; \
	fi; \
	if cargo deny --version >/dev/null 2>&1; then \
		$(CARGO) deny check > ci/stdout/amentys/deny.log 2> ci/stderr/amentys/deny.log || deny=1; \
	else \
		echo "cargo deny not installed; failing CI" > ci/stdout/amentys/deny.log; \
		echo "cargo deny not installed; failing CI" > ci/stderr/amentys/deny.log; \
		deny=1; \
	fi; \
	if [ "$$audit" -ne 0 ]; then fail_steps+=("audit"); fi; \
	if [ "$$deny" -ne 0 ]; then fail_steps+=("deny"); fi; \
	if [ "$${#fail_steps[@]}" -gt 0 ]; then \
			old_ifs="$$IFS"; \
			IFS=", "; \
			failure_summary="$${fail_steps[*]}"; \
			IFS="$$old_ifs"; \
	else \
			failure_summary=""; \
	fi; \
	if [ "$$audit" -eq 0 ] && [ "$$deny" -eq 0 ]; then \
		status_txt="$$s"; status_col="$(C_GREEN)"; \
	else \
		status_txt="$$f ($$failure_summary)"; status_col="$(C_RED)"; \
		status=1; \
	fi; \
	counter_col="$(C_GREEN)";start_col="$(C_BLUE)"; end_col="$(C_BLUE)"; version_col="$(C_CYAN)"; edition_col="$(C_CYAN)"; pkg_col="$(C_WHITE)"; desc_col="$(C_WHITE)"; \
	global_end=$$(date +'%H:%M:%S'); \
	printf "%s" "$$pad_str$(C_WHITE)"; \
	printf '%s%-5s%s %s%-73s%s %s%-5s%s %s%-5s%s %s%-5s%s %s%-5s%s %s%10s%s %s%-3s%s\n' \
		"$${counter_col}" "00/$${#pkgs[@]}" "$(C_RESET)" \
		"$${desc_col}" "Amentys" "$(C_RESET)" \
		"$${start_col}" "$$global_start" "$(C_RESET)" \
		"$${end_col}" "$$global_end" "$(C_RESET)" \
		"$${version_col}" "$$WORKSPACE_VERSION" "$(C_RESET)" \
		"$${edition_col}" "$$WORKSPACE_EDITION" "$(C_RESET)" \
		"$${pkg_col}" "amentys" "$(C_RESET)" \
		"$${status_col}" "$$status_txt" "$(C_RESET)"; \
	for i in "$${!pkgs[@]}"; do \
		failed_steps=(); \
		if [ "$$i" -lt "9" ]; then step="$(C_GREEN)0$$((i + 1))/$${#pkgs[@]}$(C_RESET)"; else step="$(C_GREEN)$$((i + 1))/$${#pkgs[@]}$(C_RESET)"; fi; \
		pkg="$${pkgs[$$i]}"; \
		version="$${versions[$$i]}"; \
		edition="$${editions[$$i]}"; \
		description="$${descs[$$i]}"; \
		mkdir -p ci/stdout/$$pkg ci/stderr/$$pkg; \
		test=0; check=0; clippy=0; fmt=0; machete=0; \
		start=$$(date +'%H:%M:%S'); \
		s="ok"; f="ko"; \
		if [ "$$pkg" == "re" ]; then flags="$(CLIPPY_KERNEL_FLAGS)"; else flags="$(CLIPPY_USER_FLAGS)"; fi; \
		if [ "$$pkg" != "re" ] && [ "$$pkg" != "maat" ] ; then  $(CARGO) test -p $$pkg --lib --release --jobs $(NUM_JOBS) > ci/stdout/$$pkg/test.log 2> ci/stderr/$$pkg/test.log || test=1; fi; \
		$(CARGO) check -p $$pkg --lib --release --jobs $(NUM_JOBS) > ci/stdout/$$pkg/check.log 2> ci/stderr/$$pkg/check.log || check=1; \
		$(CARGO) clippy -p $$pkg --release --jobs $(NUM_JOBS) -- $$flags > ci/stdout/$$pkg/clippy.log 2> ci/stderr/$$pkg/clippy.log || clippy=1; \
		$(CARGO) fmt -p $$pkg -- --check > ci/stdout/$$pkg/fmt.log 2> ci/stderr/$$pkg/fmt.log || fmt=1; \
		$(CARGO) machete $$pkg > ci/stdout/$$pkg/machete.log 2> ci/stderr/$$pkg/machete.log || machete=1; \
		if [ "$$test" -ne 0 ]; then failed_steps+=("test"); fi; \
		if [ "$$check" -ne 0 ]; then failed_steps+=("check"); fi; \
		if [ "$$clippy" -ne 0 ]; then failed_steps+=("clippy"); fi; \
		if [ "$$fmt" -ne 0 ]; then failed_steps+=("fmt"); fi; \
		if [ "$$machete" -ne 0 ]; then failed_steps+=("machete"); fi; \
		if [ "$${#failed_steps[@]}" -gt 0 ]; then \
			old_ifs="$$IFS"; \
			IFS=", "; \
			failure_summary="$${failed_steps[*]}"; \
			IFS="$$old_ifs"; \
		else \
			failure_summary=""; \
		fi; \
		end=$$(date +'%H:%M:%S'); \
		if [ "$$test" -eq 0 ] && [ "$$check" -eq 0 ] && [ "$$clippy" -eq 0 ] && [ "$$fmt" -eq 0 ] && [ "$$machete" -eq 0 ]; then \
			status_txt="$$s"; status_col="$(C_GREEN)"; \
		else \
			status_txt="$$f ($$failure_summary)"; status_col="$(C_RED)"; \
			status=1; \
		fi; \
		printf "%s" "$$pad_str";\
		printf '%s%-5s%s %s%-73s%s %s%-5s%s %s%-5s%s %s%-5s%s %s%-5s%s %s%10s%s %s%-3s%s\n' \
			"$${counter_col}" "$$step" "$(C_RESET)" \
			"$${desc_col}" "$$description" "$(C_RESET)" \
			"$${start_col}" "$$start" "$(C_RESET)" \
			"$${end_col}" "$$end" "$(C_RESET)" \
			"$${version_col}" "$$version" "$(C_RESET)" \
			"$${edition_col}" "$$edition" "$(C_RESET)"  \
			"$${pkg_col}" "$$pkg" "$(C_RESET)" \
			"$${status_col}" "$$status_txt" "$(C_RESET)"; \
	done; \
	if [ $$status -eq 0 ]; then \
		printf "\n%s" "$$pad_str$(C_WHITE)"; \
		printf '%s\n\n' "All tests passed successfully!$(C_RESET)"; \
	else \
		printf "%s" "$$pad_str$(C_KO)"; \
		printf '%s\n\n' "Some tests failed! Check the logs in ci/stdout and ci/stderr.$(C_RESET)"; \
	fi; \
	exit $$status
endef

# Terminal cursor management
define disable_cursor
	tput civis 2>/dev/null || true;
endef
# Terminal cursor management
define enable_cursor
	tput cnorm 2>/dev/null || true;
endef
