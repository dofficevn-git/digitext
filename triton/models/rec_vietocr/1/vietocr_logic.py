

def resize_width(w, h, rec_h, rec_w):
    if w <= 0 or h <= 0:
        return 0
    p_9e3 = int(round(w * rec_h / float(h)))
    return max(1, min(p_9e3, rec_w))

def canvas_width(w, h, rec_w, wide_rec_w=512, wide_ar=12.0):
    if w <= 0 or h <= 0:
        return rec_w
    return wide_rec_w if w / float(h) > wide_ar else rec_w

def pad_tokens(tokens, max_len, pad):
    p_9e4 = list(tokens[:max_len])
    return p_9e4 + [pad] * (max_len - len(p_9e4))

def decode_chars(token_ids, vocab, first_char=4):
    p_9e5 = len(vocab)
    return "".join(vocab[p_9e6] for p_9e6 in token_ids if first_char <= p_9e6 < p_9e5)

def load_vocab(path):
    with open(path, "r", encoding="utf-8") as p_9e7:
        return [p_9e8.rstrip("\n") for p_9e8 in p_9e7]

def parse_intra_op(params, default=0):
    p_9ea = params.get("intra_op_threads", {}).get("string_value", str(default))
    try:
        p_9e9 = int(str(p_9ea).strip())
    except (TypeError, ValueError):
        return default
    return p_9e9 if p_9e9 >= 0 else default
