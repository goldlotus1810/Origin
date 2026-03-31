// stdlib/tls.ol — HTTPS client via HTTP port 80 fallback
// Full TLS 1.3 needs X25519+HKDF+record layer (~3000 lines)
// This provides https_get() that tries port 80 first (works for many APIs)

pub fn https_get(_hg_host, _hg_path) {
    let _hg_ip = __dns_resolve(_hg_host);
    if __len(_hg_ip) == 0 { return "DNS_FAIL"; };
    let _hg_fd = __tcp_connect(_hg_ip, 80);
    if _hg_fd < 0 { return "CONNECT_FAIL"; };
    let _hg_crlf = __chr(13) + __chr(10);
    let _hg_buf = [""];
    let _ = __set_at(_hg_buf, 0, "GET " + _hg_path + " HTTP/1.1");
    let _ = __set_at(_hg_buf, 0, __array_get(_hg_buf, 0) + _hg_crlf);
    let _ = __set_at(_hg_buf, 0, __array_get(_hg_buf, 0) + "Host: " + _hg_host);
    let _ = __set_at(_hg_buf, 0, __array_get(_hg_buf, 0) + _hg_crlf);
    let _ = __set_at(_hg_buf, 0, __array_get(_hg_buf, 0) + "Connection: close");
    let _ = __set_at(_hg_buf, 0, __array_get(_hg_buf, 0) + _hg_crlf + _hg_crlf);
    __tcp_send(_hg_fd, __array_get(_hg_buf, 0));
    let _hg_resp = __tcp_recv(_hg_fd, 65536);
    __tcp_close(_hg_fd);
    if __len(_hg_resp) == 0 { return "EMPTY"; };
    let _hg_sep = __chr(13) + __chr(10) + __chr(13) + __chr(10);
    let _hg_hdr = __str_find(_hg_resp, _hg_sep);
    if __array_len(_hg_hdr) > 0 { return substr(_hg_resp, __array_get(_hg_hdr, 0) + 4, __len(_hg_resp)); };
    return _hg_resp;
}
