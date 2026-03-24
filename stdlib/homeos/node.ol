// homeos/node.ol — Dynamic Node + QR Record system
//
// Pipeline: UDC encode → create node → Learning → DN/QR
//
// Node = structured record:
//   { dn: sha256_address, mol: molecule, emo: {v,a}, intent, text, links[] }
//
// DN = Dynamic Name: SHA-256 hash of content → unique 64-char address
// QR = Query Record: compact format for retrieval and linking

// ════════════════════════════════════════════════════════════════
// Node Store — all learned nodes with DN addresses
// ════════════════════════════════════════════════════════════════

let __nodes = [];
let __nodes_max = 256;
let __node_count = 0;

// Create a new node from processed input
pub fn node_create(_nc_text, _nc_mol, _nc_emo, _nc_intent) {
    // DN = SHA-256 hash of text → unique address
    let _nc_dn = __sha256(_nc_text);

    // Check if node already exists (dedup by DN)
    let _nc_i = 0;
    while _nc_i < len(__nodes) {
        if __nodes[_nc_i].dn == _nc_dn {
            // Existing node → update fire count
            let _nc_existing = __nodes[_nc_i];
            set_at(__nodes, _nc_i, {
                dn: _nc_dn,
                mol: _nc_mol,
                emo: _nc_emo,
                intent: _nc_intent,
                text: _nc_text,
                fires: _nc_existing.fires + 1,
                links: _nc_existing.links
            });
            return __nodes[_nc_i];
        };
        let _nc_i = _nc_i + 1;
    };

    // New node
    let _nc_node = {
        dn: _nc_dn,
        mol: _nc_mol,
        emo: _nc_emo,
        intent: _nc_intent,
        text: _nc_text,
        fires: 1,
        links: []
    };
    push(__nodes, _nc_node);
    let __node_count = __node_count + 1;

    // Evict oldest if over limit
    if len(__nodes) > __nodes_max {
        // Keep nodes with fires > 1 (well-connected survive)
        let _nc_new = [];
        let _nc_j = 0;
        while _nc_j < len(__nodes) {
            if __nodes[_nc_j].fires > 1 {
                push(_nc_new, __nodes[_nc_j]);
            };
            let _nc_j = _nc_j + 1;
        };
        // If still too many, just keep latest half
        if len(_nc_new) > __nodes_max {
            let _nc_half = [];
            let _nc_k = __floor(len(_nc_new) / 2);
            while _nc_k < len(_nc_new) {
                push(_nc_half, _nc_new[_nc_k]);
                let _nc_k = _nc_k + 1;
            };
            let __nodes = _nc_half;
        } else {
            let __nodes = _nc_new;
        };
    };

    return _nc_node;
}

// ════════════════════════════════════════════════════════════════
// Node linking — connect related nodes
// ════════════════════════════════════════════════════════════════

pub fn node_link(_nl_dn_a, _nl_dn_b) {
    // Find both nodes and add bidirectional link
    let _nl_i = 0;
    while _nl_i < len(__nodes) {
        if __nodes[_nl_i].dn == _nl_dn_a {
            // Add link to B (if not already linked)
            let _nl_links = __nodes[_nl_i].links;
            let _nl_found = 0;
            let _nl_li = 0;
            while _nl_li < len(_nl_links) {
                if _nl_links[_nl_li] == _nl_dn_b { _nl_found = 1; };
                let _nl_li = _nl_li + 1;
            };
            if _nl_found == 0 {
                push(_nl_links, _nl_dn_b);
            };
        };
        let _nl_i = _nl_i + 1;
    };
}

// ════════════════════════════════════════════════════════════════
// QR — Query Record: search nodes by DN or content
// ════════════════════════════════════════════════════════════════

// Lookup by DN (exact)
pub fn qr_lookup(_ql_dn) {
    let _ql_i = 0;
    while _ql_i < len(__nodes) {
        if __nodes[_ql_i].dn == _ql_dn {
            return __nodes[_ql_i];
        };
        let _ql_i = _ql_i + 1;
    };
    return { dn: "", text: "", fires: 0 };
}

// Search by keyword (fuzzy)
pub fn qr_search(_qs_query) {
    let _qs_best = { dn: "", text: "", fires: 0 };
    let _qs_best_score = 0;
    let _qs_i = 0;
    while _qs_i < len(__nodes) {
        let _qs_node = __nodes[_qs_i];
        let _qs_score = _qr_match_score(_qs_query, _qs_node.text);
        // Weight by fire count (well-connected nodes rank higher)
        _qs_score = _qs_score + _qs_node.fires;
        if _qs_score > _qs_best_score {
            _qs_best_score = _qs_score;
            _qs_best = _qs_node;
        };
        let _qs_i = _qs_i + 1;
    };
    return _qs_best;
}

fn _qr_match_score(_qm_query, _qm_text) {
    // Count shared 3+ char words
    let _qm_score = 0;
    let _qm_w = "";
    let _qm_i = 0;
    while _qm_i < len(_qm_query) {
        let _qm_ch = char_at(_qm_query, _qm_i);
        if __char_code(_qm_ch) == 32 {
            if len(_qm_w) >= 3 {
                if _qr_has(_qm_text, _qm_w) == 1 {
                    _qm_score = _qm_score + 1;
                };
            };
            _qm_w = "";
        } else {
            _qm_w = _qm_w + _qm_ch;
        };
        let _qm_i = _qm_i + 1;
    };
    if len(_qm_w) >= 3 {
        if _qr_has(_qm_text, _qm_w) == 1 {
            _qm_score = _qm_score + 1;
        };
    };
    return _qm_score;
}

fn _qr_has(_qrh_text, _qrh_word) {
    let _qrh_tl = len(_qrh_text);
    let _qrh_wl = len(_qrh_word);
    if _qrh_wl > _qrh_tl { return 0; };
    let _qrh_i = 0;
    while _qrh_i <= (_qrh_tl - _qrh_wl) {
        let _qrh_m = 1;
        let _qrh_j = 0;
        while _qrh_j < _qrh_wl {
            if char_at(_qrh_text, _qrh_i + _qrh_j) != char_at(_qrh_word, _qrh_j) {
                _qrh_m = 0;
                break;
            };
            let _qrh_j = _qrh_j + 1;
        };
        if _qrh_m == 1 { return 1; };
        let _qrh_i = _qrh_i + 1;
    };
    return 0;
}

// ════════════════════════════════════════════════════════════════
// Node stats
// ════════════════════════════════════════════════════════════════

pub fn node_count() { return len(__nodes); }

pub fn node_stats() {
    let _ns_total = len(__nodes);
    let _ns_linked = 0;
    let _ns_fires = 0;
    let _ns_i = 0;
    while _ns_i < _ns_total {
        if len(__nodes[_ns_i].links) > 0 { _ns_linked = _ns_linked + 1; };
        _ns_fires = _ns_fires + __nodes[_ns_i].fires;
        let _ns_i = _ns_i + 1;
    };
    return { total: _ns_total, linked: _ns_linked, total_fires: _ns_fires };
}
