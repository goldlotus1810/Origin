// stdlib/net.ol — TCP server + client networking
// Requires VM builtins: __tcp_listen, __tcp_accept, __tcp_connect,
//   __tcp_send, __tcp_recv, __tcp_close

// ── TCP Server ──

pub fn tcp_serve(_ts_port, _ts_handler) {
    let _ts_fd = __tcp_listen(_ts_port);
    if _ts_fd < 0 { return err("listen failed"); };
    let _ts_running = 1;
    while _ts_running == 1 {
        let _ts_client = __tcp_accept(_ts_fd);
        if _ts_client >= 0 {
            let _ts_req = __tcp_recv(_ts_client, 65536);
            let _ts_resp = _ts_handler(_ts_req);
            __tcp_send(_ts_client, _ts_resp);
            __tcp_close(_ts_client);
        };
    };
    __tcp_close(_ts_fd);
    return 0;
}

// ── HTTP response helper ──

pub fn http_response(_hr_status, _hr_body) {
    let _hr_len = __to_string(len(_hr_body));
    return "HTTP/1.1 " + _hr_status + "\r\nContent-Length: " + _hr_len + "\r\nConnection: close\r\n\r\n" + _hr_body;
}

pub fn http_ok(_ho_body) {
    return http_response("200 OK", _ho_body);
}

pub fn http_404() {
    return http_response("404 Not Found", "Not Found");
}
