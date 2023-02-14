NVDS_VERSION:=6.2

LIB_INSTALL_DIR?=/opt/nvidia/deepstream/deepstream-$(NVDS_VERSION)/lib/gst-plugins/

TARGET_LIB:= libgstnvobjconv.so

all: $(TARGET_LIB)

$(TARGET_LIB) : 
	cargo build --release

install: $(TARGET_LIB)
	cp -rv ../../target/release/$(TARGET_LIB) $(LIB_INSTALL_DIR)

clean:
	rm -rf $(TARGET_LIB)