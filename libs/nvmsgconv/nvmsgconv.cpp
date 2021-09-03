/*
 * Copyright (c) 2018-2020, NVIDIA CORPORATION.  All rights reserved.
 *
 * NVIDIA Corporation and its licensors retain all intellectual property
 * and proprietary rights in and to this software, related documentation
 * and any modifications thereto.  Any use, reproduction, disclosure or
 * distribution of this software and related documentation without an express
 * license agreement from NVIDIA Corporation is strictly prohibited.
 *
 */

#include "nvmsgconv.h"
#include <json-glib/json-glib.h>
#include <uuid.h>
#include <stdlib.h>
#include <iostream>
#include <fstream>
#include <sstream>
#include <cstring>

using namespace std;

#define CHECK_ERROR(error)                       \
  if (error)                                     \
  {                                              \
    cout << "Error: " << error->message << endl; \
    goto done;                                   \
  }

struct NvDsPayloadPriv
{
};

static const gchar *
object_enum_to_str(NvDsObjectType type, gchar *objectId)
{
  switch (type)
  {
  case NVDS_OBJECT_TYPE_VEHICLE:
    return "vehicle";
  case NVDS_OBJECT_TYPE_FACE:
    return "face";
  case NVDS_OBJECT_TYPE_PERSON:
    return "person";
  case NVDS_OBJECT_TYPE_BAG:
    return "bag";
  case NVDS_OBJECT_TYPE_BICYCLE:
    return "bicycle";
  case NVDS_OBJECT_TYPE_ROADSIGN:
    return "road_sign";
  case NVDS_OBJECT_TYPE_CUSTOM:
    return "custom";
  case NVDS_OBJECT_TYPE_UNKNOWN:
    return objectId ? objectId : "unknown";
  default:
    return "unknown";
  }
}

static const gchar *
to_str(gchar *cstr)
{
  return reinterpret_cast<const gchar *>(cstr) ? cstr : "";
}

static void
generate_mask_array(NvDsEventMsgMeta *meta, JsonArray *jArray, GList *mask)
{
  unsigned int i;
  GList *l;
  stringstream ss;
  bool started = false;

  ss << meta->trackingId << "|" << g_list_length(mask);

  for (l = mask; l != NULL; l = l->next)
  {
    GArray *polygon = (GArray *)l->data;

    if (started)
      ss << "|#";

    started = true;

    for (i = 0; i < polygon->len; i++)
    {
      gdouble value = g_array_index(polygon, gdouble, i);
      ss << "|" << value;
    }
  }
  json_array_add_string_element(jArray, ss.str().c_str());
}

static gchar *
generate_message(NvDsMsg2pCtx *ctx, NvDsEvent *events, guint size)
{
  /*
  The JSON structure of the frame
  {
   "id": frame-id,
   "timestamp": "2018-04-11T04:59:59.828Z",
   "sensorId": sensor-id,
   "objects": [
      ".......object-1 attributes...........",
      ".......object-2 attributes...........",
      ".......object-3 attributes..........."
    ]
  }
  */

  JsonNode *rootNode;
  JsonObject *jobject;
  JsonArray *jArray;
  JsonArray *maskArray = NULL;
  guint i;
  stringstream ss;
  gchar *message = NULL;

  jArray = json_array_new();

  for (i = 0; i < size; i++)
  {
    GList *objectMask = NULL;

    ss.str("");
    ss.clear();

    NvDsEventMsgMeta *meta = events[i].metadata;
    ss << meta->trackingId << "|" << meta->bbox.left << "|" << meta->bbox.top
       << "|" << meta->bbox.left + meta->bbox.width << "|" << meta->bbox.top + meta->bbox.height
       << "|" << object_enum_to_str(meta->objType, meta->objectId);

    if (meta->extMsg && meta->extMsgSize)
    {
      // Attach secondary inference attributes.
      switch (meta->objType)
      {
      case NVDS_OBJECT_TYPE_VEHICLE:
      {
        NvDsVehicleObject *dsObj = (NvDsVehicleObject *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->type) << "|" << to_str(dsObj->make) << "|"
             << to_str(dsObj->model) << "|" << to_str(dsObj->color) << "|" << to_str(dsObj->license)
             << "|" << to_str(dsObj->region) << "|" << meta->confidence;
        }
      }
      break;
      case NVDS_OBJECT_TYPE_PERSON:
      {
        NvDsPersonObject *dsObj = (NvDsPersonObject *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->gender) << "|" << dsObj->age << "|"
             << to_str(dsObj->hair) << "|" << to_str(dsObj->cap) << "|" << to_str(dsObj->apparel)
             << "|" << meta->confidence;
        }
      }
      break;
      case NVDS_OBJECT_TYPE_FACE:
      {
        NvDsFaceObject *dsObj = (NvDsFaceObject *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->gender) << "|" << dsObj->age << "|"
             << to_str(dsObj->hair) << "|" << to_str(dsObj->cap) << "|" << to_str(dsObj->glasses)
             << "|" << to_str(dsObj->facialhair) << "|" << to_str(dsObj->name) << "|"
             << "|" << to_str(dsObj->eyecolor) << "|" << meta->confidence;
        }
      }
      break;
      case NVDS_OBJECT_TYPE_VEHICLE_EXT:
      {
        NvDsVehicleObjectExt *dsObj = (NvDsVehicleObjectExt *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->type) << "|" << to_str(dsObj->make) << "|"
             << to_str(dsObj->model) << "|" << to_str(dsObj->color) << "|" << to_str(dsObj->license)
             << "|" << to_str(dsObj->region) << "|" << meta->confidence;

          if (dsObj->mask)
            objectMask = dsObj->mask;
        }
      }
      break;
      case NVDS_OBJECT_TYPE_PERSON_EXT:
      {
        NvDsPersonObjectExt *dsObj = (NvDsPersonObjectExt *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->gender) << "|" << dsObj->age << "|"
             << to_str(dsObj->hair) << "|" << to_str(dsObj->cap) << "|" << to_str(dsObj->apparel)
             << "|" << meta->confidence;

          if (dsObj->mask)
            objectMask = dsObj->mask;
        }
      }
      break;
      case NVDS_OBJECT_TYPE_FACE_EXT:
      {
        NvDsFaceObjectExt *dsObj = (NvDsFaceObjectExt *)meta->extMsg;
        if (dsObj)
        {
          ss << "|#|" << to_str(dsObj->gender) << "|" << dsObj->age << "|"
             << to_str(dsObj->hair) << "|" << to_str(dsObj->cap) << "|" << to_str(dsObj->glasses)
             << "|" << to_str(dsObj->facialhair) << "|" << to_str(dsObj->name) << "|"
             << "|" << to_str(dsObj->eyecolor) << "|" << meta->confidence;

          if (dsObj->mask)
            objectMask = dsObj->mask;
        }
      }
      break;
      default:
        cout << "Object type (" << meta->objType << ") not implemented" << endl;
        break;
      }
    }

    if (objectMask)
    {
      if (maskArray == NULL)
        maskArray = json_array_new();
      generate_mask_array(meta, maskArray, objectMask);
    }

    json_array_add_string_element(jArray, ss.str().c_str());
  }

  // It is assumed that all events / objects are associated with same frame.
  // Therefore ts / sensorId / frameId of first object can be used.

  jobject = json_object_new();
  json_object_set_int_member(jobject, "id", events[0].metadata->frameId);
  json_object_set_string_member(jobject, "timestamp", events[0].metadata->ts);
  json_object_set_int_member(jobject, "sensor_id", events[0].metadata->sensorId);

  json_object_set_array_member(jobject, "objects", jArray);
  if (maskArray && json_array_get_length(maskArray) > 0)
    json_object_set_array_member(jobject, "masks", maskArray);

  rootNode = json_node_new(JSON_NODE_OBJECT);
  json_node_set_object(rootNode, jobject);

  message = json_to_string(rootNode, TRUE);
  json_node_free(rootNode);
  json_object_unref(jobject);

  return message;
}

NvDsMsg2pCtx *nvds_msg2p_ctx_create(const gchar *file, NvDsPayloadType type)
{
  NvDsMsg2pCtx *ctx = NULL;
  string str;
  bool retVal = true;

  ctx = new NvDsMsg2pCtx;
  ctx->privData = nullptr;
  ctx->payloadType = type;

  if (!retVal)
  {
    cout << "Error in creating instance" << endl;

    if (ctx && ctx->privData)
      delete (NvDsPayloadPriv *)ctx->privData;

    if (ctx)
    {
      delete ctx;
      ctx = NULL;
    }
  }
  return ctx;
}

void nvds_msg2p_ctx_destroy(NvDsMsg2pCtx *ctx)
{
  delete (NvDsPayloadPriv *)ctx->privData;
  ctx->privData = nullptr;
  delete ctx;
}

NvDsPayload **
nvds_msg2p_generate_multiple(NvDsMsg2pCtx *ctx, NvDsEvent *events, guint eventSize,
                             guint *payloadCount)
{
  gchar *message = NULL;
  gint len = 0;
  NvDsPayload **payloads = NULL;
  *payloadCount = 0;
  //Set how many payloads are being sent back to the plugin
  payloads = (NvDsPayload **)g_malloc0(sizeof(NvDsPayload *) * 1);

  message = generate_message(ctx, events, eventSize);
  if (message)
  {
    len = strlen(message);
    payloads[*payloadCount] = (NvDsPayload *)g_malloc0(sizeof(NvDsPayload));
    // Remove '\0' character at the end of string and just copy the content.
    payloads[*payloadCount]->payload = g_memdup(message, len);
    payloads[*payloadCount]->payloadSize = len;
    ++(*payloadCount);
    g_free(message);
  }

  return payloads;
}

NvDsPayload *
nvds_msg2p_generate(NvDsMsg2pCtx *ctx, NvDsEvent *events, guint size)
{
  gchar *message = NULL;
  gint len = 0;
  NvDsPayload *payload = (NvDsPayload *)g_malloc0(sizeof(NvDsPayload));

  message = generate_message(ctx, events, size);
  if (message)
  {
    len = strlen(message);
    // Remove '\0' character at the end of string and just copy the content.
    payload->payload = g_memdup(message, len);
    payload->payloadSize = len;
    g_free(message);
  }

  return payload;
}

void nvds_msg2p_release(NvDsMsg2pCtx *ctx, NvDsPayload *payload)
{
  g_free(payload->payload);
  g_free(payload);
}
