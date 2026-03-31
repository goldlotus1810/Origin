// stdlib/http.ol — HTTP/1.1 client (Olang, uses native TCP builtins)
// Requires: __tcp_connect, __tcp_send, __tcp_recv, __tcp_close, __chr

let __http_crlf = __chr(13) + __chr(10);

// ── Simple crawl: hostname + path → body string ──
// Handles DNS + TCP + HTTP + header stripping
pub fn crawl(_cr_host, _cr_path) {
    let _cr_ip = __dns_resolve(_cr_host);
    if __len(_cr_ip) == 0 { return ""; };
    let _cr_fd = __tcp_connect(_cr_ip, 80);
    if _cr_fd < 0 { return ""; };
    // Build request: store in array[0] to avoid boot-function shadow bug
    let _cr_buf = [""];
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + "GET " + _cr_path);
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + " HTTP/1.1" + __http_crlf);
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + "Host: " + _cr_host);
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + __http_crlf);
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + "Connection: close");
    let _ = __set_at(_cr_buf, 0, __array_get(_cr_buf, 0) + __http_crlf + __http_crlf);
    __tcp_send(_cr_fd, __array_get(_cr_buf, 0));
    let _cr_resp = __tcp_recv(_cr_fd, 65536);
    __tcp_close(_cr_fd);
    if __len(_cr_resp) == 0 { return ""; };
    let _cr_hdr = __str_find(_cr_resp, __http_crlf + __http_crlf);
    if __array_len(_cr_hdr) == 0 { return _cr_resp; };
    let _cr_bstart = __array_get(_cr_hdr, 0) + 4;
    return substr(_cr_resp, _cr_bstart, __len(_cr_resp));
}

// ── Crawl + parse JSON ──
pub fn crawl_json(_cj_host, _cj_path) {
    let _cj_body = crawl(_cj_host, _cj_path);
    if __len(_cj_body) == 0 { return []; };
    return json_parse(_cj_body);
}

// ── Crawl + ingest to KnowTree ──
pub fn crawl_and_learn(_cal_host, _cal_path) {
    let _cal_body = crawl(_cal_host, _cal_path);
    if __len(_cal_body) == 0 { return "Error: empty response"; };
    return kt_ingest_full(_cal_body);
}

pub fn http_get(_hg_ip, _hg_port, _hg_path, _hg_host) {
    let _hg_fd = __tcp_connect(_hg_ip, _hg_port);
    if _hg_fd < 0 { return { status: 0, body: "connection failed" }; };
    let _hg_req = "GET " + _hg_path + " HTTP/1.1" + __http_crlf + "Host: " + _hg_host + __http_crlf + "Connection: close" + __http_crlf + __http_crlf;
    let _hg_sent = __tcp_send(_hg_fd, _hg_req);
    let _hg_resp = __tcp_recv(_hg_fd, 65536);
    __tcp_close(_hg_fd);
    // Parse status code from "HTTP/1.1 200 OK\r\n..."
    let _hg_status = 0;
    if __len(_hg_resp) > 12 {
        let _hg_s1 = __char_code(char_at(_hg_resp, 9)) - 48;
        let _hg_s2 = __char_code(char_at(_hg_resp, 10)) - 48;
        let _hg_s3 = __char_code(char_at(_hg_resp, 11)) - 48;
        let _hg_status = _hg_s1 * 100 + _hg_s2 * 10 + _hg_s3;
    };
    // Find body after \r\n\r\n
    let _hg_body = _hg_resp;
    let _hg_hdr_end = __str_find(_hg_resp, __http_crlf + __http_crlf);
    let _hg_hlen = __array_len(_hg_hdr_end);
    if _hg_hlen > 0 { let _hg_bstart = __array_get(_hg_hdr_end, 0) + 4; let _hg_body = substr(_hg_resp, _hg_bstart, __len(_hg_resp)); };
    return { status: _hg_status, body: _hg_body };
}

pub fn http_post(_hp_ip, _hp_port, _hp_path, _hp_host, _hp_body) {
    let _hp_fd = __tcp_connect(_hp_ip, _hp_port);
    if _hp_fd < 0 { return { status: 0, body: "connection failed" }; };
    let _hp_req = "POST " + _hp_path + " HTTP/1.1" + __http_crlf + "Host: " + _hp_host + __http_crlf + "Content-Length: " + to_string(__len(_hp_body)) + __http_crlf + "Connection: close" + __http_crlf + __http_crlf + _hp_body;
    __tcp_send(_hp_fd, _hp_req);
    let _hp_resp = __tcp_recv(_hp_fd, 65536);
    __tcp_close(_hp_fd);
    let _hp_status = 0;
    if __len(_hp_resp) > 12 { let _hp_status = (__char_code(char_at(_hp_resp, 9))-48)*100 + (__char_code(char_at(_hp_resp, 10))-48)*10 + (__char_code(char_at(_hp_resp, 11))-48); };
    let _hp_rbody = _hp_resp;
    let _hp_hdr_end = __str_find(_hp_resp, __http_crlf + __http_crlf);
    if __array_len(_hp_hdr_end) > 0 { let _hp_bstart = __array_get(_hp_hdr_end, 0) + 4; let _hp_rbody = substr(_hp_resp, _hp_bstart, __len(_hp_resp)); };
    return { status: _hp_status, body: _hp_rbody };
}
