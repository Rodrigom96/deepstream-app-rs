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

#include <gst/gst.h>
#include <gst/base/gstbasetransform.h>
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include "gstnvobjconv.h"
#include "nvdsmeta_schema.h"
#include "nvdsmeta.h"
#include "gstnvdsmeta.h"

GST_DEBUG_CATEGORY_STATIC(gst_nvobjconv_debug_category);
#define GST_CAT_DEFAULT gst_nvobjconv_debug_category

#define MAX_TIME_STAMP_LEN 32

#define PGIE_CLASS_ID_VEHICLE 0
#define PGIE_CLASS_ID_PERSON 2

static void gst_nvobjconv_set_property(GObject *object,
                                       guint property_id, const GValue *value, GParamSpec *pspec);
static void gst_nvobjconv_get_property(GObject *object,
                                       guint property_id, GValue *value, GParamSpec *pspec);
static void gst_nvobjconv_dispose(GObject *object);
static void gst_nvobjconv_finalize(GObject *object);
static gboolean gst_nvobjconv_set_caps(GstBaseTransform *trans,
                                       GstCaps *incaps, GstCaps *outcaps);
static gboolean gst_nvobjconv_start(GstBaseTransform *trans);
static gboolean gst_nvobjconv_stop(GstBaseTransform *trans);
static GstFlowReturn gst_nvobjconv_transform_ip(GstBaseTransform *trans,
                                                GstBuffer *buf);

enum
{
  PROP_0
};

static GstStaticPadTemplate gst_nvobjconv_src_template =
    GST_STATIC_PAD_TEMPLATE("src",
                            GST_PAD_SRC,
                            GST_PAD_ALWAYS,
                            GST_STATIC_CAPS_ANY);

static GstStaticPadTemplate gst_nvobjconv_sink_template =
    GST_STATIC_PAD_TEMPLATE("sink",
                            GST_PAD_SINK,
                            GST_PAD_ALWAYS,
                            GST_STATIC_CAPS_ANY);

G_DEFINE_TYPE_WITH_CODE(GstNvObjConv, gst_nvobjconv, GST_TYPE_BASE_TRANSFORM,
                        GST_DEBUG_CATEGORY_INIT(gst_nvobjconv_debug_category, "nvobjconv", 0,
                                                "debug category for nvobjconv element"));

static void
gst_nvobjconv_class_init(GstNvObjConvClass *klass)
{
  GObjectClass *gobject_class = G_OBJECT_CLASS(klass);
  GstBaseTransformClass *base_transform_class =
      GST_BASE_TRANSFORM_CLASS(klass);

  gst_element_class_add_static_pad_template(GST_ELEMENT_CLASS(klass),
                                            &gst_nvobjconv_src_template);
  gst_element_class_add_static_pad_template(GST_ELEMENT_CLASS(klass),
                                            &gst_nvobjconv_sink_template);

  gst_element_class_set_static_metadata(GST_ELEMENT_CLASS(klass),
                                        "Objects Converter", "Filter/Metadata",
                                        "Transforms buffer objects to meta",
                                        "Rodrigom");

  gobject_class->set_property = gst_nvobjconv_set_property;
  gobject_class->get_property = gst_nvobjconv_get_property;
  gobject_class->dispose = gst_nvobjconv_dispose;
  gobject_class->finalize = gst_nvobjconv_finalize;
  base_transform_class->set_caps = GST_DEBUG_FUNCPTR(gst_nvobjconv_set_caps);
  base_transform_class->start = GST_DEBUG_FUNCPTR(gst_nvobjconv_start);
  base_transform_class->stop = GST_DEBUG_FUNCPTR(gst_nvobjconv_stop);
  base_transform_class->transform_ip =
      GST_DEBUG_FUNCPTR(gst_nvobjconv_transform_ip);
}

static void
gst_nvobjconv_init(GstNvObjConv *self)
{
  self->stop = FALSE;
  self->dsMetaQuark = g_quark_from_static_string(NVDS_META_STRING);
  self->frame_number = 0;

  gst_base_transform_set_passthrough(GST_BASE_TRANSFORM(self), TRUE);
}

void gst_nvobjconv_set_property(GObject *object, guint property_id,
                                const GValue *value, GParamSpec *pspec)
{
  GstNvObjConv *self = GST_NVOBJCONV(object);

  GST_DEBUG_OBJECT(self, "set_property");

  switch (property_id)
  {
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

void gst_nvobjconv_get_property(GObject *object, guint property_id,
                                GValue *value, GParamSpec *pspec)
{
  GstNvObjConv *self = GST_NVOBJCONV(object);

  GST_DEBUG_OBJECT(self, "get_property");

  switch (property_id)
  {
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

void gst_nvobjconv_dispose(GObject *object)
{
  GstNvObjConv *self = GST_NVOBJCONV(object);

  GST_DEBUG_OBJECT(self, "dispose");

  G_OBJECT_CLASS(gst_nvobjconv_parent_class)->dispose(object);
}

void gst_nvobjconv_finalize(GObject *object)
{
  GstNvObjConv *self = GST_NVOBJCONV(object);

  GST_DEBUG_OBJECT(self, "finalize");

  G_OBJECT_CLASS(gst_nvobjconv_parent_class)->finalize(object);
}

static gboolean
gst_nvobjconv_set_caps(GstBaseTransform *trans, GstCaps *incaps,
                       GstCaps *outcaps)
{
  GstNvObjConv *self = GST_NVOBJCONV(trans);

  GST_DEBUG_OBJECT(self, "set_caps");

  return TRUE;
}

static gboolean
gst_nvobjconv_start(GstBaseTransform *trans)
{
  GstNvObjConv *self = GST_NVOBJCONV(trans);
  gchar *error;

  GST_DEBUG_OBJECT(self, "start");

  self->stop = FALSE;

  return TRUE;
}

static gboolean
gst_nvobjconv_stop(GstBaseTransform *trans)
{
  GstNvObjConv *self = GST_NVOBJCONV(trans);

  GST_DEBUG_OBJECT(self, "stop");

  self->stop = TRUE;

  g_object_unref((GObject *)self);

  return TRUE;
}

static void generate_ts_rfc3339(char *buf, int buf_size)
{
  time_t tloc;
  struct tm tm_log;
  struct timespec ts;
  char strmsec[6]; //.nnnZ\0

  clock_gettime(CLOCK_REALTIME, &ts);
  memcpy(&tloc, (void *)(&ts.tv_sec), sizeof(time_t));
  gmtime_r(&tloc, &tm_log);
  strftime(buf, buf_size, "%Y-%m-%dT%H:%M:%S", &tm_log);
  int ms = ts.tv_nsec / 1000000;
  g_snprintf(strmsec, sizeof(strmsec), ".%.3dZ", ms);
  strncat(buf, strmsec, buf_size);
}

static gpointer meta_copy_func(gpointer data, gpointer user_data)
{
  NvDsUserMeta *user_meta = (NvDsUserMeta *)data;
  NvDsEventMsgMeta *srcMeta = (NvDsEventMsgMeta *)user_meta->user_meta_data;
  NvDsEventMsgMeta *dstMeta = NULL;

  dstMeta = g_memdup(srcMeta, sizeof(NvDsEventMsgMeta));

  if (srcMeta->ts)
    dstMeta->ts = g_strdup(srcMeta->ts);

  if (srcMeta->sensorStr)
    dstMeta->sensorStr = g_strdup(srcMeta->sensorStr);

  if (srcMeta->objSignature.size > 0)
  {
    dstMeta->objSignature.signature = g_memdup(srcMeta->objSignature.signature,
                                               srcMeta->objSignature.size);
    dstMeta->objSignature.size = srcMeta->objSignature.size;
  }

  if (srcMeta->objectId)
  {
    dstMeta->objectId = g_strdup(srcMeta->objectId);
  }

  if (srcMeta->extMsgSize > 0)
  {
    if (srcMeta->objType == NVDS_OBJECT_TYPE_VEHICLE)
    {
      NvDsVehicleObject *srcObj = (NvDsVehicleObject *)srcMeta->extMsg;
      NvDsVehicleObject *obj = (NvDsVehicleObject *)g_malloc0(sizeof(NvDsVehicleObject));
      if (srcObj->type)
        obj->type = g_strdup(srcObj->type);
      if (srcObj->make)
        obj->make = g_strdup(srcObj->make);
      if (srcObj->model)
        obj->model = g_strdup(srcObj->model);
      if (srcObj->color)
        obj->color = g_strdup(srcObj->color);
      if (srcObj->license)
        obj->license = g_strdup(srcObj->license);
      if (srcObj->region)
        obj->region = g_strdup(srcObj->region);

      dstMeta->extMsg = obj;
      dstMeta->extMsgSize = sizeof(NvDsVehicleObject);
    }
    else if (srcMeta->objType == NVDS_OBJECT_TYPE_PERSON)
    {
      NvDsPersonObject *srcObj = (NvDsPersonObject *)srcMeta->extMsg;
      NvDsPersonObject *obj = (NvDsPersonObject *)g_malloc0(sizeof(NvDsPersonObject));

      obj->age = srcObj->age;

      if (srcObj->gender)
        obj->gender = g_strdup(srcObj->gender);
      if (srcObj->cap)
        obj->cap = g_strdup(srcObj->cap);
      if (srcObj->hair)
        obj->hair = g_strdup(srcObj->hair);
      if (srcObj->apparel)
        obj->apparel = g_strdup(srcObj->apparel);
      dstMeta->extMsg = obj;
      dstMeta->extMsgSize = sizeof(NvDsPersonObject);
    }
  }

  return dstMeta;
}

static void meta_free_func(gpointer data, gpointer user_data)
{
  NvDsUserMeta *user_meta = (NvDsUserMeta *)data;
  NvDsEventMsgMeta *srcMeta = (NvDsEventMsgMeta *)user_meta->user_meta_data;

  g_free(srcMeta->ts);
  g_free(srcMeta->sensorStr);

  if (srcMeta->objSignature.size > 0)
  {
    g_free(srcMeta->objSignature.signature);
    srcMeta->objSignature.size = 0;
  }

  if (srcMeta->objectId)
  {
    g_free(srcMeta->objectId);
  }

  if (srcMeta->extMsgSize > 0)
  {
    if (srcMeta->objType == NVDS_OBJECT_TYPE_VEHICLE)
    {
      NvDsVehicleObject *obj = (NvDsVehicleObject *)srcMeta->extMsg;
      if (obj->type)
        g_free(obj->type);
      if (obj->color)
        g_free(obj->color);
      if (obj->make)
        g_free(obj->make);
      if (obj->model)
        g_free(obj->model);
      if (obj->license)
        g_free(obj->license);
      if (obj->region)
        g_free(obj->region);
    }
    else if (srcMeta->objType == NVDS_OBJECT_TYPE_PERSON)
    {
      NvDsPersonObject *obj = (NvDsPersonObject *)srcMeta->extMsg;

      if (obj->gender)
        g_free(obj->gender);
      if (obj->cap)
        g_free(obj->cap);
      if (obj->hair)
        g_free(obj->hair);
      if (obj->apparel)
        g_free(obj->apparel);
    }
    g_free(srcMeta->extMsg);
    srcMeta->extMsgSize = 0;
  }
  g_free(user_meta->user_meta_data);
  user_meta->user_meta_data = NULL;
}

static void
generate_vehicle_meta(gpointer data)
{
  NvDsVehicleObject *obj = (NvDsVehicleObject *)data;

  obj->type = g_strdup("sedan");
  obj->color = g_strdup("blue");
  obj->make = g_strdup("Bugatti");
  obj->model = g_strdup("M");
  obj->license = g_strdup("XX1234");
  obj->region = g_strdup("CA");
}

static void
generate_person_meta(gpointer data)
{
  NvDsPersonObject *obj = (NvDsPersonObject *)data;
  obj->age = 45;
  obj->cap = g_strdup("none");
  obj->hair = g_strdup("black");
  obj->gender = g_strdup("male");
  obj->apparel = g_strdup("formal");
}

static void
generate_event_msg_meta(gpointer data, gint class_id, guint sensor_id, NvDsObjectMeta *obj_params)
{
  NvDsEventMsgMeta *meta = (NvDsEventMsgMeta *)data;
  meta->sensorId = sensor_id;
  meta->placeId = sensor_id;
  meta->moduleId = sensor_id;

  meta->ts = (gchar *)g_malloc0(MAX_TIME_STAMP_LEN + 1);
  meta->objectId = (gchar *)g_malloc0(MAX_LABEL_SIZE);

  strncpy(meta->objectId, obj_params->obj_label, MAX_LABEL_SIZE);

  generate_ts_rfc3339(meta->ts, MAX_TIME_STAMP_LEN);

  if (class_id == PGIE_CLASS_ID_VEHICLE)
  {
    meta->type = NVDS_EVENT_MOVING;
    meta->objType = NVDS_OBJECT_TYPE_VEHICLE;
    meta->objClassId = PGIE_CLASS_ID_VEHICLE;

    NvDsVehicleObject *obj = (NvDsVehicleObject *)g_malloc0(sizeof(NvDsVehicleObject));
    generate_vehicle_meta(obj);

    meta->extMsg = obj;
    meta->extMsgSize = sizeof(NvDsVehicleObject);
  }
  else if (class_id == PGIE_CLASS_ID_PERSON)
  {
    meta->type = NVDS_EVENT_ENTRY;
    meta->objType = NVDS_OBJECT_TYPE_PERSON;
    meta->objClassId = PGIE_CLASS_ID_PERSON;

    NvDsPersonObject *obj = (NvDsPersonObject *)g_malloc0(sizeof(NvDsPersonObject));
    generate_person_meta(obj);

    meta->extMsg = obj;
    meta->extMsgSize = sizeof(NvDsPersonObject);
  }
}

static GstFlowReturn
gst_nvobjconv_transform_ip(GstBaseTransform *trans, GstBuffer *buf)
{
  GstNvObjConv *self = GST_NVOBJCONV(trans);
  NvDsMeta *meta = NULL;
  NvDsBatchMeta *batch_meta = NULL;
  GstMeta *gstMeta = NULL;
  gpointer state = NULL;

  GST_DEBUG_OBJECT(self, "transform_ip");

  while ((gstMeta = gst_buffer_iterate_meta(buf, &state)))
  {
    if (gst_meta_api_type_has_tag(gstMeta->info->api, self->dsMetaQuark))
    {
      meta = (NvDsMeta *)gstMeta;
      if (meta->meta_type == NVDS_BATCH_GST_META)
      {
        batch_meta = (NvDsBatchMeta *)meta->meta_data;
        break;
      }
    }
  }

  if (batch_meta)
  {
    NvDsMetaList *l = NULL;
    NvDsMetaList *l_frame = NULL;
    NvDsMetaList *l_obj = NULL;
    NvDsMetaList *user_meta_list = NULL;
    NvDsFrameMeta *frame_meta = NULL;
    NvDsUserMeta *user_event_meta = NULL;

    for (l_frame = batch_meta->frame_meta_list; l_frame; l_frame = l_frame->next)
    {
      frame_meta = (NvDsFrameMeta *)l_frame->data;

      if (frame_meta == NULL)
      {
        continue;
      }

      for (l_obj = frame_meta->obj_meta_list; l_obj; l_obj = l_obj->next)
      {
        NvDsObjectMeta *obj_meta = (NvDsObjectMeta *)l_obj->data;

        if (obj_meta == NULL)
        {
          continue;
        }

        NvDsEventMsgMeta *msg_meta = (NvDsEventMsgMeta *)g_malloc0(sizeof(NvDsEventMsgMeta));
        msg_meta->bbox.top = obj_meta->rect_params.top;
        msg_meta->bbox.left = obj_meta->rect_params.left;
        msg_meta->bbox.width = obj_meta->rect_params.width;
        msg_meta->bbox.height = obj_meta->rect_params.height;
        msg_meta->frameId = self->frame_number;
        msg_meta->trackingId = obj_meta->object_id;
        msg_meta->confidence = obj_meta->confidence;
        generate_event_msg_meta(msg_meta, obj_meta->class_id, frame_meta->source_id, obj_meta);

        NvDsUserMeta *user_event_meta = nvds_acquire_user_meta_from_pool(batch_meta);
        if (user_event_meta)
        {
          user_event_meta->user_meta_data = (void *)msg_meta;
          user_event_meta->base_meta.meta_type = NVDS_EVENT_MSG_META;
          user_event_meta->base_meta.copy_func = (NvDsMetaCopyFunc)meta_copy_func;
          user_event_meta->base_meta.release_func = (NvDsMetaReleaseFunc)meta_free_func;
          nvds_add_user_meta_to_frame(frame_meta, user_event_meta);
        }
        else
        {
          g_print("Error in attaching event meta to buffer\n");
        }
      }
    }
  }
  self->frame_number++;

  return GST_FLOW_OK;
}

static gboolean
plugin_init(GstPlugin *plugin)
{
  return gst_element_register(plugin, "nvobjconv", GST_RANK_NONE,
                              GST_TYPE_NVOBJCONV);
}

#define PACKAGE "nvobjconv"

GST_PLUGIN_DEFINE(GST_VERSION_MAJOR,
                  GST_VERSION_MINOR,
                  nvdsgst_objconv,
                  "Objects conversion",
                  plugin_init, DS_VERSION, "Proprietary", "NvObjConv", "http://nvidia.com")
