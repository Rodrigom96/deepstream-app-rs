LIB_INSTALL_DIR?=/opt/nvidia/deepstream/deepstream/lib/gst-plugins/

TARGET_LIB:= libgstnvobjconv.so

all: $(TARGET_LIB)

$(TARGET_LIB) : 
	cargo build --release

install: $(TARGET_LIB)
	cp -rv ../../target/release/$(TARGET_LIB) $(LIB_INSTALL_DIR)

clean:
	rm -rf $(TARGET_LIB)