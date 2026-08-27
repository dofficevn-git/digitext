

def resize_width(w, h, rec_h, rec_w):
    if w <= 0 or h <= 0:
        return 0
    p_a45 = int(round(w * rec_h / float(h)))
    return max(1, min(p_a45, rec_w))

def canvas_width(w, h, rec_w, wide_rec_w=512, wide_ar=12.0):
    if w <= 0 or h <= 0:
        return rec_w
    return wide_rec_w if w / float(h) > wide_ar else rec_w

def pad_tokens(tokens, max_len, pad):
    p_a46 = list(tokens[:max_len])
    return p_a46 + [pad] * (max_len - len(p_a46))

def decode_chars(token_ids, vocab, first_char=4):
    p_a47 = len(vocab)
    return "".join(vocab[p_a48] for p_a48 in token_ids if first_char <= p_a48 < p_a47)

def load_vocab(path):
    with open(path, "r", encoding="utf-8") as p_a49:
        return [p_a4a.rstrip("\n") for p_a4a in p_a49]

def parse_intra_op(params, default=0):
    p_a4c = params.get("intra_op_threads", {}).get("string_value", str(default))
    try:
        p_a4b = int(str(p_a4c).strip())
    except (TypeError, ValueError):
        return default
    return p_a4b if p_a4b >= 0 else default
