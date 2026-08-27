
import json
import os
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
        p_a09 = ort.SessionOptions()
        p_a07 = json.loads(args.get("model_config", "{}")).get("parameters", {})
        p_a09.intra_op_num_threads = L.parse_intra_op(p_a07, 0)
        p_a08 = ["CPUExecutionProvider"]
        self.sess = {}
        for p_a0b, p_a0a in ((REC_W, "w256"), (WIDE_REC_W, "w512")):
            p_a06 = os.path.join(self.model_dir, p_a0a)
            self.sess[p_a0b] = (
                ort.InferenceSession(os.path.join(p_a06, "encoder.onnx"), p_a09, providers=p_a08),
                ort.InferenceSession(os.path.join(p_a06, "decoder.onnx"), p_a09, providers=p_a08),
            )

    def execute(self, requests):
        if not self.sess or cv2 is None:
            raise pb_utils.TritonModelException("cmVjX3ZpZXRvY3I6IG9ubnhydW50aW1lL2N2Mi9tb2RlbCBjaMawYSBz4bq1biBzw6BuZw==")
        p_a0d = []
        for p_a0c in requests:
            img = pb_utils.get_input_tensor_by_name(p_a0c, "image").as_numpy()
            text, score = self._decode(img)
            p_a0d.append(
                pb_utils.InferenceResponse(
                    output_tensors=[
                        pb_utils.Tensor(
                            "text", np.array([text.encode("utf-8")], dtype=np.object_)
                        ),
                        pb_utils.Tensor(
                            "score", np.array([score], dtype=np.float32)
                        ),
                    ]
                )
            )
        return p_a0d

    def _preprocess(self, img):
        p_a0e = np.asarray(img, dtype=np.float32)
        if p_a0e.ndim == 4:
            p_a0e = p_a0e[0]
        if p_a0e.ndim == 2:
            p_a0e = np.repeat(p_a0e[:, :, None], 3, axis=2)
        p_a10, p_a13 = int(p_a0e.shape[0]), int(p_a0e.shape[1])
        p_a12 = L.canvas_width(p_a13, p_a10, REC_W, WIDE_REC_W)
        p_a0f = np.full((REC_H, p_a12, 3), 255.0, dtype=np.float32)
        p_a11 = L.resize_width(p_a13, p_a10, REC_H, p_a12)
        if p_a11 > 0:
            p_a0f[:, :p_a11] = cv2.resize(p_a0e, (p_a11, REC_H))
        p_a14 = (p_a0f / 255.0).transpose(2, 0, 1)[None]
        return np.ascontiguousarray(p_a14, dtype=np.float32), p_a12

    def _decode(self, img):
        p_a1e, p_a1b = self._preprocess(img)
        p_a18, p_a17 = self.sess[p_a1b]
        memory = p_a18.run(None, {"image": p_a1e})[0]
        p_a1d = [SOS]
        p_a1a = []
        for p_a15 in range(MAX_LEN - 1):
            p_a16 = np.array([L.pad_tokens(p_a1d, MAX_LEN, PAD)], dtype=np.int64)
            logits = p_a17.run(None, {"memory": memory, "tgt": p_a16})[0]
            p_a1c = logits[0, len(p_a1d) - 1]
            p_a1a.append(_softmax_max(p_a1c))
            p_a19 = int(p_a1c.argmax())
            if p_a19 == EOS:
                break
            p_a1d.append(p_a19)
        text = unicodedata.normalize("NFC", L.decode_chars(p_a1d[1:], self.vocab))
        score = float(np.mean(p_a1a)) if p_a1a else 0.0
        return text, score

def _softmax_max(logits):
    p_a20 = float(np.max(logits))
    p_a1f = np.exp(logits - p_a20)
    return float(np.max(p_a1f) / np.sum(p_a1f))
