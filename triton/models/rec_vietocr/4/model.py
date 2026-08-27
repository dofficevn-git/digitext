
import json
import os
import sys
import unicodedata

import numpy as np
import triton_python_backend_utils as pb_utils

import vietocr_logic as L

try:
    import cv2
except ImportError:
    cv2 = None
try:
    import onnxruntime as ort
except ImportError:
    ort = None

REC_H, REC_W, MAX_LEN = 32, 256, 128
WIDE_REC_W = 512
PAD, SOS, EOS = 0, 1, 2

class TritonPythonModel:
    def initialize(self, args):
        self.model_dir = os.path.dirname(__file__)
        self.vocab = L.load_vocab(os.path.join(self.model_dir, "vocab.txt"))
        self.sess = {}
        if ort is None:
            return
        p_a2d = ort.SessionOptions()
        p_a2a = json.loads(args.get("model_config", "{}")).get("parameters", {})
        p_a2d.intra_op_num_threads = L.parse_intra_op(p_a2a, 0)
        p_a2b = ["CUDAExecutionProvider", "CPUExecutionProvider"]
        for p_a2f, p_a2e in ((REC_W, "w256"), (WIDE_REC_W, "w512")):
            p_a29 = os.path.join(self.model_dir, p_a2e)
            self.sess[p_a2f] = (
                ort.InferenceSession(os.path.join(p_a29, "encoder.onnx"), p_a2d, providers=p_a2b),
                ort.InferenceSession(os.path.join(p_a29, "decoder.onnx"), p_a2d, providers=p_a2b),
            )
        p_a2c = self.sess[REC_W][0].get_providers()[0]
        if p_a2c != "CUDAExecutionProvider":
            print(
                f"WARN rec_vietocr v4: provider THỰC TẾ = {p_a2c} (rớt về CPU) — thiếu "
                "onnxruntime-gpu/CUDA trong python backend; MẤT mức tăng GPU, health vẫn xanh.",
                file=sys.stderr, flush=True,
            )
        else:
            print("INFO rec_vietocr v4: CUDAExecutionProvider hoạt động.", file=sys.stderr, flush=True)

    def execute(self, requests):
        if not self.sess or cv2 is None:
            raise pb_utils.TritonModelException("rec_vietocr: onnxruntime/cv2/model chưa sẵn sàng")
        p_a31 = []
        for p_a30 in requests:
            img = pb_utils.get_input_tensor_by_name(p_a30, "image").as_numpy()
            text, score = self._decode(img)
            p_a31.append(
                pb_utils.InferenceResponse(
                    output_tensors=[
                        pb_utils.Tensor("text", np.array([text.encode("utf-8")], dtype=np.object_)),
                        pb_utils.Tensor("score", np.array([score], dtype=np.float32)),
                    ]
                )
            )
        return p_a31

    def _preprocess(self, img):
        p_a32 = np.asarray(img, dtype=np.float32)
        if p_a32.ndim == 4:
            p_a32 = p_a32[0]
        if p_a32.ndim == 2:
            p_a32 = np.repeat(p_a32[:, :, None], 3, axis=2)
        p_a34, p_a37 = int(p_a32.shape[0]), int(p_a32.shape[1])
        p_a36 = L.canvas_width(p_a37, p_a34, REC_W, WIDE_REC_W)
        p_a33 = np.full((REC_H, p_a36, 3), 255.0, dtype=np.float32)
        p_a35 = L.resize_width(p_a37, p_a34, REC_H, p_a36)
        if p_a35 > 0:
            p_a33[:, :p_a35] = cv2.resize(p_a32, (p_a35, REC_H))
        p_a38 = (p_a33 / 255.0).transpose(2, 0, 1)[None]
        return np.ascontiguousarray(p_a38, dtype=np.float32), p_a36

    def _decode(self, img):
        p_a42, p_a3f = self._preprocess(img)
        p_a3c, p_a3b = self.sess[p_a3f]
        memory = p_a3c.run(None, {"image": p_a42})[0]
        p_a41 = [SOS]
        p_a3e = []
        for p_a39 in range(MAX_LEN - 1):
            p_a3a = np.array([L.pad_tokens(p_a41, MAX_LEN, PAD)], dtype=np.int64)
            logits = p_a3b.run(None, {"memory": memory, "tgt": p_a3a})[0]
            p_a40 = logits[0, len(p_a41) - 1]
            p_a3e.append(_softmax_max(p_a40))
            p_a3d = int(p_a40.argmax())
            if p_a3d == EOS:
                break
            p_a41.append(p_a3d)
        text = unicodedata.normalize("NFC", L.decode_chars(p_a41[1:], self.vocab))
        score = float(np.mean(p_a3e)) if p_a3e else 0.0
        return text, score

def _softmax_max(logits):
    p_a44 = float(np.max(logits))
    p_a43 = np.exp(logits - p_a44)
    return float(np.max(p_a43) / np.sum(p_a43))
