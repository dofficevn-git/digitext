#!/usr/bin/env python3
import argparse
import os

import torch
import torch.nn as nn

from vietocr.tool.config import Cfg
from vietocr.tool.predictor import Predictor

REC_H = 32

class q_1(nn.Module):

    def __init__(self, model):
        super().__init__()
        self.model = model

    def forward(self, image):
        return self.model.transformer.forward_encoder(self.model.cnn(image))

class q_0(nn.Module):

    def __init__(self, model):
        super().__init__()
        self.model = model

    def forward(self, memory, tgt):
        p_8a4 = tgt.transpose(0, 1).contiguous()
        out, p_8a3 = self.model.transformer.forward_decoder(p_8a4, memory)
        return out

def apply_weights(cfg, weights):
    cfg["device"] = "cpu"
    cfg["predictor"]["beamsearch"] = False
    if weights:
        cfg["weights"] = weights
    return cfg

def q_d6(path):
    import onnx

    p_8a7 = onnx.load(path)
    p_8a6 = 0
    for p_8a9 in p_8a7.graph.node:
        if p_8a9.op_type != "Transpose":
            continue
        for p_8a5 in p_8a9.attribute:
            if p_8a5.name == "perm" and any(p_8aa < 0 for p_8aa in p_8a5.ints):
                p_8a8 = [p_8aa if p_8aa >= 0 else p_8aa + len(p_8a5.ints) for p_8aa in p_8a5.ints]
                del p_8a5.ints[:]
                p_8a5.ints.extend(p_8a8)
                p_8a6 += 1
    if p_8a6:
        onnx.save(p_8a7, path)
    print(f"    {os.path.basename(path)}: sửa {p_8a6} Transpose perm âm")

def main():
    p_8ac = argparse.ArgumentParser()
    p_8ac.add_argument("--out", required=True)
    p_8ac.add_argument("--name", default="vgg_transformer")
    p_8ac.add_argument("--opset", type=int, default=17)
    p_8ac.add_argument("--max-len", type=int, default=128, help="cửa sổ giải mã cố định")
    p_8ac.add_argument("--rec-w", type=int, default=256, help="bề rộng crop CỐ ĐỊNH lúc infer")
    p_8ac.add_argument("--weights", default=None,
                    help="checkpoint VietOCR fine-tune (bỏ trống = pretrained gốc)")
    p_8ad = p_8ac.parse_args()
    os.makedirs(p_8ad.out, exist_ok=True)

    cfg = apply_weights(Cfg.load_config_from_name(p_8ad.name), p_8ad.weights)
    print(f"· tải VietOCR '{p_8ad.name}'"
          f"{f' + weights {p_8ad.weights}' if p_8ad.weights else ' (pretrained)'}…", flush=True)
    predictor = Predictor(cfg)
    model = predictor.model.eval()
    vocab = predictor.vocab

    p_8bb = len(vocab)
    with open(os.path.join(p_8ad.out, "vocab.txt"), "w", encoding="utf-8") as p_8b5:
        p_8b5.write("\n".join(str(vocab.i2c.get(p_8b6, "*")) for p_8b6 in range(p_8bb)) + "\n")
    print(f"· vocab.txt: {p_8bb} token (pad={vocab.pad} SOS={vocab.go} EOS={vocab.eos})")
    assert vocab.pad == 0 and vocab.go == 1 and vocab.eos == 2, "PAD/SOS/EOS phải =0/1/2"

    import numpy as np
    import onnxruntime as ort

    p_8b2 = q_1(model).eval()
    p_8af = q_0(model).eval()
    p_8b3 = os.path.join(p_8ad.out, "encoder.onnx")
    p_8b0 = os.path.join(p_8ad.out, "decoder.onnx")

    p_8b8 = torch.randn(1, 3, REC_H, p_8ad.rec_w)
    with torch.no_grad():
        memory = p_8b2(p_8b8)
    torch.onnx.export(p_8b2, (p_8b8,), p_8b3,
                      input_names=["image"], output_names=["memory"],
                      dynamic_axes={"image": {3: "W"}, "memory": {0: "S"}},
                      opset_version=p_8ad.opset)
    q_d6(p_8b3)
    print(f"· encoder.onnx ✓ (W động)  memory{tuple(memory.shape)}")

    tgt = torch.full((1, p_8ad.max_len), vocab.pad, dtype=torch.long)
    tgt[0, 0] = vocab.go
    with torch.no_grad():
        logits = p_8af(memory, tgt)
    torch.onnx.export(p_8af, (memory, tgt), p_8b0,
                      input_names=["memory", "tgt"], output_names=["logits"],
                      dynamic_axes={"memory": {0: "S"}},
                      opset_version=p_8ad.opset)
    q_d6(p_8b0)
    print(f"· decoder.onnx ✓ (T={p_8ad.max_len} cố định, S động)  logits{tuple(logits.shape)}")

    p_8b4 = ort.InferenceSession(p_8b3, providers=["CPUExecutionProvider"])
    p_8b1 = ort.InferenceSession(p_8b0, providers=["CPUExecutionProvider"])

    def greedy(width):
        p_8b7 = torch.randn(1, 3, REC_H, width).numpy().astype(np.float32)
        p_8ba = p_8b4.run(None, {"image": p_8b7})[0]
        p_8bf = [vocab.go]
        p_8ae = np.full((1, p_8ad.max_len), vocab.pad, dtype=np.int64)
        for p_8ab in range(p_8ad.max_len - 1):
            p_8ae[0, : len(p_8bf)] = p_8bf
            p_8b9 = p_8b1.run(None, {"memory": p_8ba, "tgt": p_8ae})[0]
            p_8bd = int(p_8b9[0, len(p_8bf) - 1].argmax())
            p_8bf.append(p_8bd)
            if p_8bd == vocab.eos:
                break
        return len(p_8bf), p_8ba.shape[0]

    p_8bc, p_8be = greedy(p_8ad.rec_w)
    print(f"· ORT greedy W={p_8ad.rec_w} (S={p_8be}): {p_8bc} token OK")

    with open(os.path.join(p_8ad.out, "export_meta.txt"), "w") as p_8b5:
        p_8b5.write(f"MAX_LEN={p_8ad.max_len}\nREC_H={REC_H}\nREC_W={p_8ad.rec_w}\nVOCAB={p_8bb}\n"
                f"PAD={vocab.pad}\nSOS={vocab.go}\nEOS={vocab.eos}\n")
    print(f"✅ encoder.onnx + decoder.onnx + vocab.txt sẵn sàng "
          f"(REC_W={p_8ad.rec_w} MAX_LEN={p_8ad.max_len})")

if __name__ == "__main__":
    main()
