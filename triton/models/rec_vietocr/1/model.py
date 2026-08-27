
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
        p_9d2 = ort.SessionOptions()
        p_9d0 = json.loads(args.get("model_config", "{}")).get("parameters", {})
        p_9d2.intra_op_num_threads = L.parse_intra_op(p_9d0, 0)
        p_9d1 = ["CPUExecutionProvider"]
        self.enc = ort.InferenceSession(
            os.path.join(self.model_dir, "encoder.onnx"), p_9d2, providers=p_9d1
        )
        self.dec = ort.InferenceSession(
            os.path.join(self.model_dir, "decoder.onnx"), p_9d2, providers=p_9d1
        )

    def execute(self, requests):
        if self.enc is None or self.dec is None or cv2 is None:
            raise pb_utils.TritonModelException("cmVjX3ZpZXRvY3I6IG9ubnhydW50aW1lL2N2Mi9tb2RlbCBjaMawYSBz4bq1biBzw6BuZw==")
        p_9d4 = []
        for p_9d3 in requests:
            img = pb_utils.get_input_tensor_by_name(p_9d3, "image").as_numpy()
            text, score = self._decode(img)
            p_9d4.append(
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
        return p_9d4

    def _preprocess(self, img):
        p_9d5 = np.asarray(img, dtype=np.float32)
        if p_9d5.ndim == 4:
            p_9d5 = p_9d5[0]
        if p_9d5.ndim == 2:
            p_9d5 = np.repeat(p_9d5[:, :, None], 3, axis=2)
        p_9d7, p_9d9 = int(p_9d5.shape[0]), int(p_9d5.shape[1])
        p_9d6 = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
        p_9d8 = L.resize_width(p_9d9, p_9d7, REC_H, REC_W)
        if p_9d8 > 0:
            p_9d6[:, :p_9d8] = cv2.resize(p_9d5, (p_9d8, REC_H))
        p_9da = (p_9d6 / 255.0).transpose(2, 0, 1)[None]
        return np.ascontiguousarray(p_9da, dtype=np.float32)

    def _decode(self, img):
        memory = self.enc.run(None, {"image": self._preprocess(img)})[0]
        p_9e0 = [SOS]
        p_9de = []
        for p_9db in range(MAX_LEN - 1):
            p_9dc = np.array([L.pad_tokens(p_9e0, MAX_LEN, PAD)], dtype=np.int64)
            logits = self.dec.run(None, {"memory": memory, "tgt": p_9dc})[0]
            p_9df = logits[0, len(p_9e0) - 1]
            p_9de.append(_softmax_max(p_9df))
            p_9dd = int(p_9df.argmax())
            if p_9dd == EOS:
                break
            p_9e0.append(p_9dd)
        text = unicodedata.normalize("NFC", L.decode_chars(p_9e0[1:], self.vocab))
        score = float(np.mean(p_9de)) if p_9de else 0.0
        return text, score

def _softmax_max(logits):
    p_9e2 = float(np.max(logits))
    p_9e1 = np.exp(logits - p_9e2)
    return float(np.max(p_9e1) / np.sum(p_9e1))
