
import json
import os
import unicodedata

import cv2
import numpy as np
import triton_python_backend_utils as pb_utils

import ocr_logic

class TritonPythonModel:
    def initialize(self, args):
        p_938 = json.loads(args.get("model_config", "{}")).get("parameters", {})
        self.conf_fallback = float(
            p_938.get("conf_fallback", {}).get("string_value", "0.80")
        )
        self.svtr_enabled = ocr_logic.parse_bool(
            p_938.get("svtr_enabled", {}).get("string_value", "false"), default=False
        )
        self.det_thresh = 0.30
        self.nms_iou = 0.30
        self.rec_buckets = [160, 320, 480, 640]
        self.model_dir = os.path.dirname(__file__)
        self.charset = self._load_charset(os.path.join(self.model_dir, "dict_VI.txt"))

    @staticmethod
    def _load_charset(path):
        if not os.path.exists(path):
            return None
        with open(path, "r", encoding="utf-8") as p_939:
            return [None] + [p_93a.rstrip("\n") for p_93a in p_939]

    def execute(self, requests):
        p_945 = []
        for p_944 in requests:
            tile = pb_utils.get_input_tensor_by_name(p_944, "tile").as_numpy()
            p_93f, p_94a = tile.shape[0], tile.shape[1]

            det_out = self._infer(
                "det_dbnet", "x", self._preprocess_det_glue(tile), "sigmoid_0.tmp_0"
            )
            p_93e = self._postprocess_det_glue(det_out, self.det_thresh)

            p_93b = [ocr_logic.clamp_box(p_93d[1], p_94a, p_93f) for p_93d in p_93e]
            p_947 = [p_93d[2] for p_93d in p_93e]
            p_942 = ocr_logic.nms(p_93b, p_947, self.nms_iou)
            p_943 = ocr_logic.sort_reading_order(
                [p_93b[p_941] for p_941 in p_942], row_tol=max(1, p_93f // 40)
            )
            p_942 = [p_942[p_940] for p_940 in p_943]

            boxes, texts, p_946 = [], [], []
            for p_941 in p_942:
                crop = self._crop_rotate_glue(tile, p_93e[p_941][0])
                if self.svtr_enabled:
                    p_93c = ocr_logic.bucket_width(crop.shape[1], self.rec_buckets)
                    rec_in = self._bucket_resize_glue(crop, p_93c)
                    text, score = self._recognize_svtr(rec_in)
                    if ocr_logic.needs_fallback(score, self.conf_fallback):
                        p_949, p_948 = self._recognize_vietocr(crop)
                        text, score = ocr_logic.pick_best((text, score), (p_949, p_948))
                else:
                    text, score = self._recognize_vietocr(crop)
                boxes.append(p_93b[p_941])
                texts.append(text)
                p_946.append(score)

            p_945.append(
                pb_utils.InferenceResponse(
                    output_tensors=[
                        pb_utils.Tensor(
                            "boxes", np.array(boxes, dtype=np.float32).reshape(-1, 4)
                        ),
                        pb_utils.Tensor("texts", np.array(texts, dtype=object)),
                        pb_utils.Tensor("confidences", np.array(p_946, dtype=np.float32)),
                    ]
                )
            )
        return p_945

    def _infer(self, model, in_name, in_tensor, out_name):
        p_94b = pb_utils.InferenceRequest(
            model_name=model,
            requested_output_names=[out_name],
            inputs=[pb_utils.Tensor(in_name, in_tensor)],
            preferred_memory=pb_utils.PreferredMemory(pb_utils.TRITONSERVER_MEMORY_CPU, 0),
        )
        p_94c = p_94b.exec()
        if p_94c.has_error():
            raise pb_utils.TritonModelException(p_94c.error().message())
        return pb_utils.get_output_tensor_by_name(p_94c, out_name).as_numpy()

    def _recognize_svtr(self, rec_in):
        logits = self._infer("rec_svtr_vi", "x", rec_in, "ctc_probs")
        return self._ctc_to_text(logits)

    def _ctc_to_text(self, logits):
        p_952 = np.asarray(logits, dtype=np.float32).reshape(logits.shape[-2], logits.shape[-1])
        p_950 = p_952.argmax(axis=-1).tolist()
        p_951 = p_952.max(axis=-1).tolist()
        p_94e, score = ocr_logic.ctc_greedy_decode_conf(p_950, p_951, blank_id=0)
        if self.charset is None:
            raise pb_utils.TritonModelException("dGhp4bq/dSBkaWN0X1ZJLnR4dCBj4bqhbmggbW9kZWwucHkgKMKnUzEyLjQp")
        p_94d = [self.charset[p_94f] for p_94f in p_94e if 0 < p_94f < len(self.charset)]
        text = unicodedata.normalize("NFC", "".join(p_94d))
        return text, float(score)

    def _recognize_vietocr(self, crop):
        p_953 = pb_utils.InferenceRequest(
            model_name="rec_vietocr",
            requested_output_names=["text", "score"],
            inputs=[pb_utils.Tensor("image", crop.astype(np.float32))],
            preferred_memory=pb_utils.PreferredMemory(pb_utils.TRITONSERVER_MEMORY_CPU, 0),
        )
        p_954 = p_953.exec()
        if p_954.has_error():
            raise pb_utils.TritonModelException(p_954.error().message())
        text = pb_utils.get_output_tensor_by_name(p_954, "text").as_numpy()[0]
        score = float(pb_utils.get_output_tensor_by_name(p_954, "score").as_numpy()[0])
        return q_ba(text), score

    DET_SIZE = 1024
    DET_MEAN = np.array([0.485, 0.456, 0.406], dtype=np.float32)
    DET_STD = np.array([0.229, 0.224, 0.225], dtype=np.float32)
    BOX_SCORE_THRESH = 0.5
    UNCLIP_RATIO = 1.5
    UNCLIP_Y_RATIO = 0.4
    REC_H = 48

    def _preprocess_det_glue(self, tile):
        self._det_src_hw = (int(tile.shape[0]), int(tile.shape[1]))
        p_956 = cv2.resize(tile, (self.DET_SIZE, self.DET_SIZE)).astype(np.float32) / 255.0
        p_955 = (p_956 - self.DET_MEAN) / self.DET_STD
        return np.ascontiguousarray(p_955.transpose(2, 0, 1)[None], dtype=np.float32)

    def _postprocess_det_glue(self, det_out, thresh):
        prob = np.asarray(det_out, dtype=np.float32).reshape(self.DET_SIZE, self.DET_SIZE)
        p_959 = (prob > thresh).astype(np.uint8)
        p_95b, p_957 = cv2.findContours(p_959, cv2.RETR_LIST, cv2.CHAIN_APPROX_SIMPLE)
        p_960, p_961 = self._det_src_hw
        p_962, p_963 = p_961 / self.DET_SIZE, p_960 / self.DET_SIZE

        p_95c = []
        for p_95a in p_95b[:1000]:
            if cv2.contourArea(p_95a) < 4:
                continue
            p_95e = cv2.boxPoints(cv2.minAreaRect(p_95a))
            p_95f = ocr_logic.order_points_clockwise([(float(p_95d[0]), float(p_95d[1])) for p_95d in p_95e])
            score = self._box_score(prob, p_95f)
            if score < self.BOX_SCORE_THRESH:
                continue
            quad = self._unclip(p_95f)
            if quad is None:
                continue
            quad = ocr_logic.clip_quad([(x * p_962, p_965 * p_963) for x, p_965 in quad], p_961, p_960)
            p_964, p_966 = [p_95d[0] for p_95d in quad], [p_95d[1] for p_95d in quad]
            p_958 = (min(p_964), min(p_966), max(p_964), max(p_966))
            p_95c.append((quad, p_958, float(score)))
        return p_95c

    def _unclip(self, quad):
        return ocr_logic.unclip_quad(quad, self.UNCLIP_RATIO, self.UNCLIP_Y_RATIO)

    def _box_score(self, prob, quad):
        p_967, p_96c = prob.shape
        p_96f = [int(round(p_969[0])) for p_969 in quad]
        p_972 = [int(round(p_969[1])) for p_969 in quad]
        p_96d, p_96e = max(0, min(p_96f)), min(p_96c - 1, max(p_96f))
        p_970, p_971 = max(0, min(p_972)), min(p_967 - 1, max(p_972))
        if p_96e <= p_96d or p_971 <= p_970:
            return 0.0
        p_968 = np.zeros((p_971 - p_970 + 1, p_96e - p_96d + 1), dtype=np.uint8)
        p_96a = np.array([[int(round(p_969[0])) - p_96d, int(round(p_969[1])) - p_970] for p_969 in quad], dtype=np.int32)
        cv2.fillPoly(p_968, [p_96a], 1)
        p_96b = prob[p_970 : p_971 + 1, p_96d : p_96e + 1]
        return float(p_96b[p_968 == 1].mean()) if int(p_968.sum()) > 0 else 0.0

    def _crop_rotate_glue(self, tile, quad):
        p_977, p_975 = ocr_logic.quad_size(quad)
        p_977, p_975 = max(1, int(round(p_977))), max(1, int(round(p_975)))
        p_976 = np.array(quad, dtype=np.float32)
        p_974 = np.array([[0, 0], [p_977, 0], [p_977, p_975], [0, p_975]], dtype=np.float32)
        crop = cv2.warpPerspective(tile, cv2.getPerspectiveTransform(p_976, p_974), (p_977, p_975))
        if p_975 > p_977 * 1.5:
            crop = cv2.rotate(crop, cv2.ROTATE_90_CLOCKWISE)
        p_973 = self._infer("cls_angle", "x", self._cls_preprocess(crop), "softmax")
        if int(np.asarray(p_973).reshape(-1).argmax()) == 1:
            crop = cv2.rotate(crop, cv2.ROTATE_180)
        return crop

    def _cls_preprocess(self, crop):
        p_979 = cv2.resize(crop, (192, self.REC_H)).astype(np.float32) / 255.0
        p_978 = (p_979 - 0.5) / 0.5
        return np.ascontiguousarray(p_978.transpose(2, 0, 1)[None], dtype=np.float32)

    def _bucket_resize_glue(self, crop, bucket_w):
        p_97c, p_980 = int(crop.shape[0]), int(crop.shape[1])
        p_97f, p_97a = ocr_logic.rec_resize_target(p_980, p_97c, self.REC_H, self.rec_buckets)
        p_97f = max(1, min(p_97f, bucket_w))
        p_97e = cv2.resize(crop, (p_97f, self.REC_H))
        p_97b = np.zeros((self.REC_H, bucket_w, 3), dtype=np.uint8)
        p_97b[:, :p_97f] = p_97e
        p_97d = (p_97b.astype(np.float32) / 255.0 - 0.5) / 0.5
        return np.ascontiguousarray(p_97d.transpose(2, 0, 1)[None], dtype=np.float32)

def q_ba(value):
    if isinstance(value, (bytes, bytearray)):
        return value.decode("utf-8", "replace")
    return str(value)
