

def resize_width(w, h, rec_h, rec_w):
    if w <= 0 or h <= 0:
        return 0
    p_a21 = int(round(w * rec_h / float(h)))
    return max(1, min(p_a21, rec_w))

def canvas_width(w, h, rec_w, wide_rec_w=512, wide_ar=12.0):
    if w <= 0 or h <= 0:
        return rec_w
    return wide_rec_w if w / float(h) > wide_ar else rec_w

def pad_tokens(tokens, max_len, pad):
    p_a22 = list(tokens[:max_len])
    return p_a22 + [pad] * (max_len - len(p_a22))

def decode_chars(token_ids, vocab, first_char=4):
    p_a23 = len(vocab)
    return "".join(vocab[p_a24] for p_a24 in token_ids if first_char <= p_a24 < p_a23)

def load_vocab(path):
    with open(path, "r", encoding="utf-8") as p_a25:
        return [p_a26.rstrip("\n") for p_a26 in p_a25]

def parse_intra_op(params, default=0):
    p_a28 = params.get("intra_op_threads", {}).get("string_value", str(default))
    try:
        p_a27 = int(str(p_a28).strip())
    except (TypeError, ValueError):
        return default
    return p_a27 if p_a27 >= 0 else default
