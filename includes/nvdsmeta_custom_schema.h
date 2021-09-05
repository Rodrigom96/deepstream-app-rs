#pragma once

#include <glib.h>

#ifdef __cplusplus
extern "C"
{
#endif

    /**
 * Defines event type flags.
 */
    typedef enum NvDsEventType
    {
        NVDS_EVENT_ENTRY,
        NVDS_EVENT_EXIT,
        NVDS_EVENT_MOVING,
        NVDS_EVENT_STOPPED,
        NVDS_EVENT_EMPTY,
        NVDS_EVENT_PARKED,
        NVDS_EVENT_RESET,

        /** Reserved for future use. Custom events must be assigned values
   greater than this. */
        NVDS_EVENT_RESERVED = 0x100,
        /** Specifies a custom event. */
        NVDS_EVENT_CUSTOM = 0x101,
        NVDS_EVENT_FORCE32 = 0x7FFFFFFF
    } NvDsEventType;

    /**
 * Defines payload type flags.
 */
    typedef enum NvDsPayloadType
    {
        NVDS_PAYLOAD_DEEPSTREAM,
        NVDS_PAYLOAD_DEEPSTREAM_MINIMAL,
        /** Reserved for future use. Custom payloads must be assigned values
   greater than this. */
        NVDS_PAYLOAD_RESERVED = 0x100,
        /** Specifies a custom payload. You must implement the nvds_msg2p_*
   interface. */
        NVDS_PAYLOAD_CUSTOM = 0x101,
        NVDS_PAYLOAD_FORCE32 = 0x7FFFFFFF
    } NvDsPayloadType;

    typedef struct NvDsRect
    {
        float top;    /**< Holds the position of rectangle's top in pixels. */
        float left;   /**< Holds the position of rectangle's left side in pixels. */
        float width;  /**< Holds the rectangle's width in pixels. */
        float height; /**< Holds the rectangle's height in pixels. */
    } NvDsRect;

    typedef struct NvDsEventMsgMeta
    {
        /** Holds the object's bounding box. */
        NvDsRect bbox;
        /** Holds the object's class ID. */
        gint objClassId;
        /** Holds a pointer to a string containing the object class label. */
        gchar *objClassLabel;
        /** Holds the ID of the sensor that generated the event. */
        gint sensorId;
        /** Holds the video frame ID of this event. */
        gint frameId;
        /** Holds the confidence level of the inference. */
        gdouble confidence;
        /** Holds the object's tracking ID. */
        gint trackingId;
        /** Holds a pointer to the generated event's timestamp. */
        gchar *ts;
    } NvDsEventMsgMeta;

    /**
 * Holds event information.
 */
    typedef struct _NvDsEvent
    {
        /** Holds the type of event. */
        NvDsEventType eventType;
        /** Holds a pointer to event metadata. */
        NvDsEventMsgMeta *metadata;
    } NvDsEvent;

    /**
 * Holds payload metadata.
 */
    typedef struct NvDsPayload
    {
        /** Holds a pointer to the payload. */
        gpointer payload;
        /** Holds the size of the payload. */
        guint payloadSize;
        /** Holds the ID of the component (plugin) which attached the payload
   (optional). */
        guint componentId;
    } NvDsPayload;

#ifdef __cplusplus
}
#endif
