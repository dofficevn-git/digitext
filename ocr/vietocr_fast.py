"""Giải mã tham lam VietOCR có KV cache — thay `vietocr.tool.translate.translate` khi
OCR_REC_FAST_DECODE=true.

Vì sao: bản gốc mỗi bước chạy lại TOÀN BỘ decoder trên cả chuỗi đã sinh (chi phí ~ bình
phương độ dài dòng), softmax trên cả chuỗi rồi chép cả khối N×T×vocab về CPU chỉ để lấy ký tự
cuối. Đo 2/10/2026 trên RTX 4090 laptop: ~28 ms/dòng, và fp16 không nhanh hơn chút nào — thời
gian nằm ở vòng lặp chứ không ở phép tính.

Bản này: memory của encoder chiếu sang K/V cross-attention MỘT lần; self-attention giữ K/V các
bước trước; mỗi bước chỉ tính token mới, chỉ lấy max/argmax trên GPU.

GIỮ ĐÚNG hành vi bản gốc (độ tin cậy đang dùng cho hàng soát và ngưỡng 0,85 sang Qwen):
- dừng khi MỌI dòng đã có EOS hoặc quá max_seq_length + 1 bước — dòng xong sớm vẫn sinh tiếp
  cùng cả mẻ;
- độ tin cậy = trung bình xác suất max của mọi token > 3 trong cả chuỗi, KỂ CẢ token sinh sau
  EOS (quirk của bản gốc — vocab.decode cắt ở EOS nhưng xác suất thì không);
- cùng trọng số, cùng phép tính (post-norm, ReLU), chỉ khác thứ tự cộng số thực.
"""
from __future__ import annotations

import math

import numpy as np
import torch
import torch.nn.functional as F


def _heads(x: torch.Tensor, h: int) -> torch.Tensor:
    """(T, N, E) → (N, h, T, d)."""
    t, n, e = x.shape
    return x.view(t, n, h, e // h).permute(1, 2, 0, 3)


def _merge(x: torch.Tensor) -> torch.Tensor:
    """(N, h, T, d) → (T, N, E)."""
    n, h, t, d = x.shape
    return x.permute(2, 0, 1, 3).reshape(t, n, h * d)


def supported(model) -> bool:
    """Chỉ thay khi đúng kiến trúc đã kiểm: nn.Transformer post-norm, không batch_first."""
    try:
        lt = model.transformer
        layers = lt.transformer.decoder.layers
        return all(not getattr(l, "norm_first", False) for l in layers) and \
            not getattr(layers[0].self_attn, "batch_first", False)
    except AttributeError:
        return False


def translate(img, model, max_seq_length=128, sos_token=1, eos_token=2):
    """Cùng chữ ký và đầu ra với vietocr.tool.translate.translate: (ndarray N×T, ndarray N)."""
    model.eval()
    with torch.no_grad():
        lt = model.transformer
        dec = lt.transformer.decoder
        layers = dec.layers
        e = lt.d_model
        h = layers[0].self_attn.num_heads
        memory = lt.forward_encoder(model.cnn(img))  # S×N×E
        n = memory.shape[1]

        cross = []
        for lyr in layers:
            w, b = lyr.multihead_attn.in_proj_weight, lyr.multihead_attn.in_proj_bias
            k = F.linear(memory, w[e:2 * e], None if b is None else b[e:2 * e])
            v = F.linear(memory, w[2 * e:], None if b is None else b[2 * e:])
            cross.append((_heads(k, h), _heads(v, h)))
        cache_k = [None] * len(layers)
        cache_v = [None] * len(layers)

        pe = lt.pos_enc.pe  # max_len × 1 × E
        scale = math.sqrt(e)
        tok = torch.full((n,), sos_token, dtype=torch.long, device=memory.device)
        toks = [tok]
        probs = [torch.ones(n, device=memory.device)]
        done = torch.zeros(n, dtype=torch.bool, device=memory.device)
        step = 0
        while step <= max_seq_length and not bool(done.all()):
            x = (lt.embed_tgt(tok) * scale + pe[step, 0]).unsqueeze(0)  # 1×N×E
            for i, lyr in enumerate(layers):
                sa = lyr.self_attn
                q, k, v = F.linear(x, sa.in_proj_weight, sa.in_proj_bias).chunk(3, dim=-1)
                q, k, v = _heads(q, h), _heads(k, h), _heads(v, h)
                cache_k[i] = k if cache_k[i] is None else torch.cat([cache_k[i], k], dim=2)
                cache_v[i] = v if cache_v[i] is None else torch.cat([cache_v[i], v], dim=2)
                a = F.scaled_dot_product_attention(q, cache_k[i], cache_v[i])
                x = lyr.norm1(x + sa.out_proj(_merge(a)))

                ca = lyr.multihead_attn
                w, b = ca.in_proj_weight, ca.in_proj_bias
                q = _heads(F.linear(x, w[:e], None if b is None else b[:e]), h)
                a = F.scaled_dot_product_attention(q, cross[i][0], cross[i][1])
                x = lyr.norm2(x + ca.out_proj(_merge(a)))

                x = lyr.norm3(x + lyr.linear2(lyr.activation(lyr.linear1(x))))
            if dec.norm is not None:
                x = dec.norm(x)
            p = F.softmax(lt.fc(x[0]).float(), dim=-1)
            val, tok = p.max(dim=-1)
            toks.append(tok)
            probs.append(val)
            done |= tok == eos_token
            step += 1

        translated_sentence = torch.stack(toks, dim=1).cpu().numpy()
        char_probs = torch.stack(probs, dim=1).cpu().numpy()
        char_probs = np.multiply(char_probs, translated_sentence > 3)
        char_probs = np.sum(char_probs, axis=-1) / (char_probs > 0).sum(-1)
    return translated_sentence, char_probs


def install(predictor) -> bool:
    """Thay hàm translate mà Predictor.predict/predict_batch gọi. Trả False nếu không hỗ trợ."""
    if not supported(predictor.model):
        return False
    import vietocr.tool.predictor as vp
    vp.translate = translate
    return True
