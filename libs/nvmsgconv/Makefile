NVDS_VERSION:=5.1

LIB_INSTALL_DIR?=/opt/nvidia/deepstream/deepstream-$(NVDS_VERSION)/lib/

TARGET_LIB:= libnvds_msgconv.so

all: $(TARGET_LIB)

$(TARGET_LIB) : 
	cargo build --release

install: $(TARGET_LIB)
	cp -rv target/release/$(TARGET_LIB) $(LIB_INSTALL_DIR)

clean:
	rm -rf $(TARGET_LIB)