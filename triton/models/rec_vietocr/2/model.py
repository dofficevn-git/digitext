
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
PAD, SOS, EOS = 0, 1, 2

class TritonPythonModel:
    def initialize(self, args):
        self.model_dir = os.path.dirname(__file__)
        self.vocab = L.load_vocab(os.path.join(self.model_dir, "vocab.txt"))
        if ort is None:
            self.enc = self.dec = None
            return
        p_9ed = ort.SessionOptions()
        p_9eb = json.loads(args.get("model_config", "{}")).get("parameters", {})
        p_9ed.intra_op_num_threads = L.parse_intra_op(p_9eb, 0)
        p_9ec = ["CPUExecutionProvider"]
        self.enc = ort.InferenceSession(
            os.path.join(self.model_dir, "encoder.onnx"), p_9ed, providers=p_9ec
        )
        self.dec = ort.InferenceSession(
            os.path.join(self.model_dir, "decoder.onnx"), p_9ed, providers=p_9ec
        )

    def execute(self, requests):
        if self.enc is None or self.dec is None or cv2 is None:
            raise pb_utils.TritonModelException("cmVjX3ZpZXRvY3I6IG9ubnhydW50aW1lL2N2Mi9tb2RlbCBjaMawYSBz4bq1biBzw6BuZw==")
        p_9ef = []
        for p_9ee in requests:
            img = pb_utils.get_input_tensor_by_name(p_9ee, "image").as_numpy()
            text, score = self._decode(img)
            p_9ef.append(
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
        return p_9ef

    def _preprocess(self, img):
        p_9f0 = np.asarray(img, dtype=np.float32)
        if p_9f0.ndim == 4:
            p_9f0 = p_9f0[0]
        if p_9f0.ndim == 2:
            p_9f0 = np.repeat(p_9f0[:, :, None], 3, axis=2)
        p_9f2, p_9f4 = int(p_9f0.shape[0]), int(p_9f0.shape[1])
        p_9f1 = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
        p_9f3 = L.resize_width(p_9f4, p_9f2, REC_H, REC_W)
        if p_9f3 > 0:
            p_9f1[:, :p_9f3] = cv2.resize(p_9f0, (p_9f3, REC_H))
        p_9f5 = (p_9f1 / 255.0).transpose(2, 0, 1)[None]
        return np.ascontiguousarray(p_9f5, dtype=np.float32)

    def _decode(self, img):
        memory = self.enc.run(None, {"image": self._preprocess(img)})[0]
        p_9fb = [SOS]
        p_9f9 = []
        for p_9f6 in range(MAX_LEN - 1):
            p_9f7 = np.array([L.pad_tokens(p_9fb, MAX_LEN, PAD)], dtype=np.int64)
            logits = self.dec.run(None, {"memory": memory, "tgt": p_9f7})[0]
            p_9fa = logits[0, len(p_9fb) - 1]
            p_9f9.append(_softmax_max(p_9fa))
            p_9f8 = int(p_9fa.argmax())
            if p_9f8 == EOS:
                break
            p_9fb.append(p_9f8)
        text = unicodedata.normalize("NFC", L.decode_chars(p_9fb[1:], self.vocab))
        score = float(np.mean(p_9f9)) if p_9f9 else 0.0
        return text, score

def _softmax_max(logits):
    p_9fd = float(np.max(logits))
    p_9fc = np.exp(logits - p_9fd)
    return float(np.max(p_9fc) / np.sum(p_9fc))
