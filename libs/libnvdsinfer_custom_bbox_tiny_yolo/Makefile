CUDA_VER:=11
NVDS_VERSION:=5.1

SRCFILES:=nvdsparsebbox_tiny_yolo.cpp
TARGET_LIB:=libnvdsinfer_custom_bbox_tiny_yolo.so
DEEPSTREAM_PATH:=/opt/nvidia/deepstream/deepstream-$(NVDS_VERSION)
LIB_INSTALL_DIR?=${DEEPSTREAM_PATH}/lib/

ifeq ($(CUDA_VER),)
  $(error "CUDA_VER is not set")
endif
CC:= g++

CFLAGS:= -Wall -Werror -std=c++11 -shared -fPIC
CFLAGS+= -I$(DEEPSTREAM_PATH)/sources/includes -I/usr/local/cuda-$(CUDA_VER)/include

LIBS:= -lnvinfer -lnvparsers -L/usr/local/cuda-$(CUDA_VER)/lib64
LFLAGS:= -Wl,--start-group $(LIBS) -Wl,--end-group

all: $(TARGET_LIB)

$(TARGET_LIB) : $(SRCFILES)
	$(CC) -o $@ $^ $(CFLAGS) $(LFLAGS)

install: $(TARGET_LIB)
	cp -rv $(TARGET_LIB) $(LIB_INSTALL_DIR)

clean:
	rm -rf $(TARGET_LIB)