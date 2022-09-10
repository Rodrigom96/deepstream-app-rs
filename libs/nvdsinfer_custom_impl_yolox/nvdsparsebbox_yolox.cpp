/*
 * Copyright (c) 2020, NVIDIA CORPORATION. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <algorithm>
#include <cassert>
#include <cmath>
#include <cstring>
#include <fstream>
#include <vector>
#include <omp.h>
#include "nvdsinfer_custom_impl.h"



#define DEVICE 0  // GPU id
#define NMS_THRESH 0.65
#define BBOX_CONF_THRESH 0.3


static const int INPUT_W = 640;
static const int INPUT_H = 640;
const char* INPUT_BLOB_NAME = "images";
const char* OUTPUT_BLOB_NAME = "output";

struct GridAndStride
{
    int grid0;
    int grid1;
    int stride;
};

static void generate_grids_and_stride(const int target_size, std::vector<int>& strides, std::vector<GridAndStride>& grid_strides)
{
    for (auto stride : strides)
    {
        int num_grid = target_size / stride;
        for (int g1 = 0; g1 < num_grid; g1++)
        {
            for (int g0 = 0; g0 < num_grid; g0++)
            {
                grid_strides.push_back((GridAndStride){g0, g1, stride});
            }
        }
    }
}

static float overlap1D(float x1min, float x1max, float x2min, float x2max)
{   
    if (x1min > x2min)
    {
        std::swap(x1min, x2min);
        std::swap(x1max, x2max);
    }
    return x1max < x2min ? 0 : std::min(x1max, x2max) - x2min;
}

static float computeIoU(const NvDsInferParseObjectInfo& bbox1, const NvDsInferParseObjectInfo& bbox2)
{
    float overlapX
        = overlap1D(bbox1.left, bbox1.left + bbox1.width, bbox2.left, bbox2.left + bbox2.width);
    float overlapY
        = overlap1D(bbox1.top, bbox1.top + bbox1.height, bbox2.top, bbox2.top + bbox2.height);
    float area1 = (bbox1.width) * (bbox1.height);
    float area2 = (bbox2.width) * (bbox2.height);
    float overlap2D = overlapX * overlapY;
    float u = area1 + area2 - overlap2D;
    return u == 0 ? 0 : overlap2D / u;
}

static void qsort_descent_inplace(std::vector<NvDsInferParseObjectInfo>& faceobjects, int left, int right)
{
    int i = left;
    int j = right;
    float p = faceobjects[(left + right) / 2].detectionConfidence;

    while (i <= j)
    {
        while (faceobjects[i].detectionConfidence > p)
            i++;

        while (faceobjects[j].detectionConfidence < p)
            j--;

        if (i <= j)
        {
            // swap
            std::swap(faceobjects[i], faceobjects[j]);

            i++;
            j--;
        }
    }

    #pragma omp parallel sections
    {
        #pragma omp section
        {
            if (left < j) qsort_descent_inplace(faceobjects, left, j);
        }
        #pragma omp section
        {
            if (i < right) qsort_descent_inplace(faceobjects, i, right);
        }
    }
}

static void qsort_descent_inplace(std::vector<NvDsInferParseObjectInfo>& objects)
{
    if (objects.empty())
        return;

    qsort_descent_inplace(objects, 0, objects.size() - 1);
}

static void nms_sorted_bboxes(const std::vector<NvDsInferParseObjectInfo>& faceobjects, std::vector<int>& picked, float nms_threshold)
{
    picked.clear();

    const int n = faceobjects.size();

    for (int i = 0; i < n; i++)
    {
        const NvDsInferParseObjectInfo& a = faceobjects[i];

        int keep = 1;
        for (int j = 0; j < (int)picked.size(); j++)
        {
            const NvDsInferParseObjectInfo& b = faceobjects[picked[j]];

            // intersection over union
            float iou = computeIoU(a, b);
            // float IoU = inter_area / union_area
            if (iou > nms_threshold)
                keep = 0;
        }

        if (keep)
            picked.push_back(i);
    }
}


static void generate_yolox_proposals(std::vector<GridAndStride> grid_strides, float* feat_blob, float prob_threshold, std::vector<NvDsInferParseObjectInfo>& objects)
{
    const int num_class = 80;

    const int num_anchors = grid_strides.size();

    for (int anchor_idx = 0; anchor_idx < num_anchors; anchor_idx++)
    {
        const int grid0 = grid_strides[anchor_idx].grid0;
        const int grid1 = grid_strides[anchor_idx].grid1;
        const int stride = grid_strides[anchor_idx].stride;

        const int basic_pos = anchor_idx * 85;

        // yolox/models/yolo_head.py decode logic
        float x_center = (feat_blob[basic_pos+0] + grid0) * stride;
        float y_center = (feat_blob[basic_pos+1] + grid1) * stride;
        float w = exp(feat_blob[basic_pos+2]) * stride;
        float h = exp(feat_blob[basic_pos+3]) * stride;
        float x0 = x_center - w * 0.5f;
        float y0 = y_center - h * 0.5f;

        float box_objectness = feat_blob[basic_pos+4];
        for (int class_idx = 0; class_idx < num_class; class_idx++)
        {
            float box_cls_score = feat_blob[basic_pos + 5 + class_idx];
            float box_prob = box_objectness * box_cls_score;
            if (box_prob > prob_threshold)
            {
                NvDsInferParseObjectInfo obj;
                obj.left = static_cast<unsigned int>(x0);
                obj.top = static_cast<unsigned int>(y0);
                obj.width = static_cast<unsigned int>(w);
                obj.height = static_cast<unsigned int>(h);
                obj.classId = class_idx;
                obj.detectionConfidence = box_prob;

                objects.push_back(obj);
            }

        } // class loop

    } // point anchor loop
}



static void decode_outputs(float* prob, std::vector<NvDsInferParseObjectInfo>& objects, float scale, const int img_w, const int img_h) {
        std::vector<NvDsInferParseObjectInfo> proposals;
        std::vector<int> strides = {8, 16, 32};
        std::vector<GridAndStride> grid_strides;
        generate_grids_and_stride(INPUT_W, strides, grid_strides);
        generate_yolox_proposals(grid_strides, prob,  BBOX_CONF_THRESH, proposals);
        // std::cout << "num of boxes before nms: " << proposals.size() << std::endl;

        qsort_descent_inplace(proposals);

        std::vector<int> picked;
        nms_sorted_bboxes(proposals, picked, NMS_THRESH);


        int count = picked.size();

        // std::cout << "num of boxes: " << count << std::endl;

        objects.resize(count);
        for (int i = 0; i < count; i++)
        {
            objects[i] = proposals[picked[i]];

            // adjust offset to original unpadded
            // float x0 = (objects[i].rect.x) / scale;
            // float y0 = (objects[i].rect.y) / scale;
            // float x1 = (objects[i].rect.x + objects[i].rect.width) / scale;
            // float y1 = (objects[i].rect.y + objects[i].rect.height) / scale;
            float x0 = (objects[i].left);
            float y0 = (objects[i].top);
            float x1 = (objects[i].left + objects[i].width);
            float y1 = (objects[i].top + objects[i].height);

            // clip
            x0 = std::max(std::min(x0, (float)(img_w - 1)), 0.f);
            y0 = std::max(std::min(y0, (float)(img_h - 1)), 0.f);
            x1 = std::max(std::min(x1, (float)(img_w - 1)), 0.f);
            y1 = std::max(std::min(y1, (float)(img_h - 1)), 0.f);

            objects[i].left = x0;
            objects[i].top = y0;
            objects[i].width = x1 - x0;
            objects[i].height = y1 - y0;
        }
}


/* This is a sample bounding box parsing function for the sample YoloV5 detector model */
static bool NvDsInferParseYolox(
    std::vector<NvDsInferLayerInfo> const& outputLayersInfo,
    NvDsInferNetworkInfo const& networkInfo,
    NvDsInferParseDetectionParams const& detectionParams,
    std::vector<NvDsInferParseObjectInfo>& objectList)
{
    float* prob = (float*)outputLayersInfo[0].buffer;
    std::vector<NvDsInferParseObjectInfo> objects;
    int img_w = 1920;
    int img_h = 1080;
    float scale = std::min(INPUT_W / (img_w*1.0), INPUT_H / (img_h*1.0));
    decode_outputs(prob, objects, scale, img_w, img_h);
    
    for(auto& r : objects) {
	    NvDsInferParseObjectInfo oinfo;
        
	    oinfo.classId = r.classId;
        oinfo.left    = r.left;
	    oinfo.top     = r.top;
	    oinfo.width   = r.width;
	    oinfo.height  = r.height;
	    oinfo.detectionConfidence = r.detectionConfidence;
	    objectList.push_back(oinfo);
    }
    return true;
}

extern "C" bool NvDsInferParseCustomYolox(
    std::vector<NvDsInferLayerInfo> const &outputLayersInfo,
    NvDsInferNetworkInfo const &networkInfo,
    NvDsInferParseDetectionParams const &detectionParams,
    std::vector<NvDsInferParseObjectInfo> &objectList)
{
    return NvDsInferParseYolox(
        outputLayersInfo, networkInfo, detectionParams, objectList);
}

/* Check that the custom function has been defined correctly */
CHECK_CUSTOM_PARSE_FUNC_PROTOTYPE(NvDsInferParseCustomYolox);
