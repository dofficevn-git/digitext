

def resize_width(w, h, rec_h, rec_w):
    if w <= 0 or h <= 0:
        return 0
    p_9fe = int(round(w * rec_h / float(h)))
    return max(1, min(p_9fe, rec_w))

def canvas_width(w, h, rec_w, wide_rec_w=512, wide_ar=16.0):
    if w <= 0 or h <= 0:
        return rec_w
    return wide_rec_w if w / float(h) >= wide_ar else rec_w

def pad_tokens(tokens, max_len, pad):
    p_9ff = list(tokens[:max_len])
    return p_9ff + [pad] * (max_len - len(p_9ff))

def decode_chars(token_ids, vocab, first_char=4):
    p_a00 = len(vocab)
    return "".join(vocab[p_a01] for p_a01 in token_ids if first_char <= p_a01 < p_a00)

def load_vocab(path):
    with open(path, "r", encoding="utf-8") as p_a02:
        return [p_a03.rstrip("\n") for p_a03 in p_a02]

def parse_intra_op(params, default=0):
    p_a05 = params.get("intra_op_threads", {}).get("string_value", str(default))
    try:
        p_a04 = int(str(p_a05).strip())
    except (TypeError, ValueError):
        return default
    return p_a04 if p_a04 >= 0 else default
