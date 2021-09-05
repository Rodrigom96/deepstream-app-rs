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
  guint i;
  gchar *message = NULL;

  jArray = json_array_new();

  for (i = 0; i < size; i++)
  {
    NvDsEventMsgMeta *meta = events[i].metadata;

    jobject = json_object_new();
    json_object_set_int_member(jobject, "id", meta->trackingId);
    json_object_set_int_member(jobject, "x", meta->bbox.left);
    json_object_set_int_member(jobject, "y", meta->bbox.top);
    json_object_set_int_member(jobject, "width", meta->bbox.width);
    json_object_set_int_member(jobject, "height", meta->bbox.height);
    json_object_set_string_member(jobject, "label", meta->objClassLabel);

    json_array_add_object_element(jArray, jobject);
  }

  // It is assumed that all events / objects are associated with same frame.
  // Therefore ts / sensorId / frameId of first object can be used.

  jobject = json_object_new();
  json_object_set_int_member(jobject, "frame_id", events[0].metadata->frameId);
  json_object_set_string_member(jobject, "timestamp", events[0].metadata->ts);
  json_object_set_int_member(jobject, "camera_id", events[0].metadata->sensorId);

  json_object_set_array_member(jobject, "objects", jArray);

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
