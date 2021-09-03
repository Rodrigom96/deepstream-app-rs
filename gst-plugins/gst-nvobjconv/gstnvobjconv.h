/*
 * Copyright (c) 2018-2020 NVIDIA CORPORATION.  All rights reserved.
 *
 * NVIDIA Corporation and its licensors retain all intellectual property
 * and proprietary rights in and to this software, related documentation
 * and any modifications thereto.  Any use, reproduction, disclosure or
 * distribution of this software and related documentation without an express
 * license agreement from NVIDIA Corporation is strictly prohibited.
 *
 */

#ifndef _GST_NVOBJCONV_H_
#define _GST_NVOBJCONV_H_

#include <gst/base/gstbasetransform.h>

G_BEGIN_DECLS

#define GST_TYPE_NVOBJCONV (gst_nvobjconv_get_type())
#define GST_NVOBJCONV(obj) (G_TYPE_CHECK_INSTANCE_CAST((obj), GST_TYPE_NVOBJCONV, GstNvObjConv))
#define GST_NVOBJCONV_CLASS(klass) (G_TYPE_CHECK_CLASS_CAST((klass), GST_TYPE_NVOBJCONV, GstNvObjConvClass))
#define GST_IS_NVMSGCONV(obj) (G_TYPE_CHECK_INSTANCE_TYPE((obj), GST_TYPE_NVOBJCONV))
#define GST_IS_NVMSGCONV_CLASS(obj) (G_TYPE_CHECK_CLASS_TYPE((klass), GST_TYPE_NVOBJCONV))

typedef struct _GstNvObjConv GstNvObjConv;
typedef struct _GstNvObjConvClass GstNvObjConvClass;

struct _GstNvObjConv
{
  GstBaseTransform parent;
  GQuark dsMetaQuark;
  gboolean stop;
  gint frame_number;
};

struct _GstNvObjConvClass
{
  GstBaseTransformClass parent_class;
};

GType gst_nvobjconv_get_type(void);

G_END_DECLS

#endif
