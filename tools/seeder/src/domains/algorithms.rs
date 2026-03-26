//! # algorithms — Algorithm categories and key algorithms
//!
//! Sorting, searching, graph, dynamic programming, numerical methods.
//! From Wikipedia: Danh sách thuật toán.

use super::{SeedEdge, SeedNode};

pub static ALGORITHM_NODES: &[SeedNode] = &[
    // ─── Core Concepts ──────────────────────────────────────────────────────
    SeedNode { name: "algorithm", codepoint: 0x1F4BB, aliases: &[
        "thuat-toan", "thuật toán", "algorithm", "algorithme",
    ]},
    SeedNode { name: "complexity", codepoint: 0x1D4AA, aliases: &[
        "do-phuc-tap", "độ phức tạp", "complexity", "complexité",
        "big-O", "O(n)", "time complexity",
    ]},
    SeedNode { name: "data_structure", codepoint: 0x1F4CA, aliases: &[
        "cau-truc-du-lieu", "cấu trúc dữ liệu", "data structure",
        "structure de données",
    ]},
    SeedNode { name: "recursion", codepoint: 0x1F504, aliases: &[
        "de-quy", "đệ quy", "recursion", "récursion",
    ]},
    SeedNode { name: "iteration", codepoint: 0x1F503, aliases: &[
        "lap", "lặp", "iteration", "itération",
    ]},

    // ─── Sorting ────────────────────────────────────────────────────────────
    SeedNode { name: "sorting", codepoint: 0x1F522, aliases: &[
        "sap-xep", "sắp xếp", "sorting", "tri",
    ]},
    SeedNode { name: "bubble_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-noi-bot", "sắp xếp nổi bọt", "bubble sort", "O(n²)",
    ]},
    SeedNode { name: "merge_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-tron", "sắp xếp trộn", "merge sort", "O(n log n)",
    ]},
    SeedNode { name: "quick_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-nhanh", "sắp xếp nhanh", "quicksort", "quick sort",
    ]},
    SeedNode { name: "heap_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-vun-dong", "sắp xếp vun đống", "heapsort", "heap sort",
    ]},
    SeedNode { name: "insertion_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-chen", "sắp xếp chèn", "insertion sort",
    ]},
    SeedNode { name: "radix_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-co-so", "sắp xếp cơ số", "radix sort",
    ]},
    SeedNode { name: "counting_sort", codepoint: 0x1F522, aliases: &[
        "sap-xep-dem", "sắp xếp đếm", "counting sort",
    ]},

    // ─── Searching ──────────────────────────────────────────────────────────
    SeedNode { name: "searching", codepoint: 0x1F50D, aliases: &[
        "tim-kiem", "tìm kiếm", "searching", "recherche",
    ]},
    SeedNode { name: "binary_search", codepoint: 0x1F50D, aliases: &[
        "tim-kiem-nhi-phan", "tìm kiếm nhị phân", "binary search",
        "O(log n)",
    ]},
    SeedNode { name: "linear_search", codepoint: 0x1F50D, aliases: &[
        "tim-kiem-tuyen-tinh", "tìm kiếm tuyến tính", "linear search",
    ]},
    SeedNode { name: "hash_search", codepoint: 0x1F50D, aliases: &[
        "tim-kiem-bang-bam", "tìm kiếm bảng băm", "hash search", "O(1)",
    ]},

    // ─── Graph Algorithms ───────────────────────────────────────────────────
    SeedNode { name: "graph_algorithm", codepoint: 0x1F578, aliases: &[
        "thuat-toan-do-thi", "thuật toán đồ thị", "graph algorithm",
    ]},
    SeedNode { name: "bfs", codepoint: 0x1F578, aliases: &[
        "BFS", "duyet-theo-chieu-rong", "duyệt theo chiều rộng",
        "breadth-first search",
    ]},
    SeedNode { name: "dfs", codepoint: 0x1F578, aliases: &[
        "DFS", "duyet-theo-chieu-sau", "duyệt theo chiều sâu",
        "depth-first search",
    ]},
    SeedNode { name: "dijkstra", codepoint: 0x1F578, aliases: &[
        "Dijkstra", "duong-di-ngan-nhat", "đường đi ngắn nhất",
        "shortest path",
    ]},
    SeedNode { name: "bellman_ford", codepoint: 0x1F578, aliases: &[
        "Bellman-Ford", "bellman ford",
    ]},
    SeedNode { name: "floyd_warshall", codepoint: 0x1F578, aliases: &[
        "Floyd-Warshall", "floyd warshall", "all-pairs shortest path",
    ]},
    SeedNode { name: "kruskal", codepoint: 0x1F578, aliases: &[
        "Kruskal", "cay-khung-nho-nhat", "cây khung nhỏ nhất",
        "minimum spanning tree",
    ]},
    SeedNode { name: "prim", codepoint: 0x1F578, aliases: &[
        "Prim", "prim algorithm",
    ]},
    SeedNode { name: "topological_sort", codepoint: 0x1F578, aliases: &[
        "sap-xep-topo", "sắp xếp tôpô", "topological sort",
    ]},
    SeedNode { name: "a_star", codepoint: 0x1F578, aliases: &[
        "A*", "a-star", "tim-duong-A*", "tìm đường A*",
    ]},

    // ─── Dynamic Programming ────────────────────────────────────────────────
    SeedNode { name: "dynamic_programming", codepoint: 0x1F4CA, aliases: &[
        "quy-hoach-dong", "quy hoạch động", "dynamic programming", "DP",
    ]},
    SeedNode { name: "memoization", codepoint: 0x1F4DD, aliases: &[
        "ghi-nho", "ghi nhớ", "memoization",
    ]},
    SeedNode { name: "greedy", codepoint: 0x1F4B0, aliases: &[
        "tham-lam", "tham lam", "greedy algorithm", "algorithme glouton",
    ]},
    SeedNode { name: "divide_conquer", codepoint: 0x2702, aliases: &[
        "chia-de-tri", "chia để trị", "divide and conquer",
        "diviser pour régner",
    ]},
    SeedNode { name: "backtracking", codepoint: 0x1F519, aliases: &[
        "quay-lui", "quay lui", "backtracking",
    ]},

    // ─── String Algorithms ──────────────────────────────────────────────────
    SeedNode { name: "string_matching", codepoint: 0x1F50E, aliases: &[
        "doi-sanh-chuoi", "đối sánh chuỗi", "string matching",
        "pattern matching",
    ]},
    SeedNode { name: "kmp", codepoint: 0x1F50E, aliases: &[
        "KMP", "Knuth-Morris-Pratt", "knuth morris pratt",
    ]},
    SeedNode { name: "rabin_karp", codepoint: 0x1F50E, aliases: &[
        "Rabin-Karp", "rabin karp",
    ]},

    // ─── Numerical ──────────────────────────────────────────────────────────
    SeedNode { name: "numerical_method", codepoint: 0x1F4BB, aliases: &[
        "phuong-phap-so", "phương pháp số", "numerical method",
        "méthode numérique",
    ]},
    SeedNode { name: "newton_method", codepoint: 0x1F4BB, aliases: &[
        "Newton", "phuong-phap-Newton", "phương pháp Newton",
        "Newton-Raphson",
    ]},
    SeedNode { name: "euler_method", codepoint: 0x1F4BB, aliases: &[
        "Euler", "phuong-phap-Euler", "phương pháp Euler",
    ]},
    SeedNode { name: "monte_carlo", codepoint: 0x1F3B2, aliases: &[
        "Monte-Carlo", "monte carlo", "phuong-phap-Monte-Carlo",
    ]},
    SeedNode { name: "fft", codepoint: 0x1F4BB, aliases: &[
        "FFT", "bien-doi-Fourier-nhanh", "biến đổi Fourier nhanh",
        "fast Fourier transform",
    ]},

    // ─── Machine Learning ───────────────────────────────────────────────────
    SeedNode { name: "machine_learning", codepoint: 0x1F916, aliases: &[
        "hoc-may", "học máy", "machine learning", "apprentissage automatique",
        "ML",
    ]},
    SeedNode { name: "neural_network", codepoint: 0x1F9E0, aliases: &[
        "mang-than-kinh", "mạng thần kinh", "neural network",
        "réseau de neurones", "NN",
    ]},
    SeedNode { name: "gradient_descent", codepoint: 0x2207, aliases: &[
        "ha-gradient", "hạ gradient", "gradient descent",
        "descente de gradient",
    ]},
];

pub static ALGORITHM_EDGES: &[SeedEdge] = &[
    // Sorting hierarchy
    SeedEdge { from: "bubble_sort", to: "sorting", relation: 0x01 },     // ∈
    SeedEdge { from: "merge_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "quick_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "heap_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "insertion_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "radix_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "counting_sort", to: "sorting", relation: 0x01 },
    SeedEdge { from: "merge_sort", to: "divide_conquer", relation: 0x08 }, // ← derived from
    SeedEdge { from: "quick_sort", to: "divide_conquer", relation: 0x08 },
    SeedEdge { from: "merge_sort", to: "quick_sort", relation: 0x07 },    // ≈ similar

    // Searching hierarchy
    SeedEdge { from: "binary_search", to: "searching", relation: 0x01 },
    SeedEdge { from: "linear_search", to: "searching", relation: 0x01 },
    SeedEdge { from: "hash_search", to: "searching", relation: 0x01 },

    // Graph algorithms
    SeedEdge { from: "bfs", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "dfs", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "dijkstra", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "bellman_ford", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "floyd_warshall", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "kruskal", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "prim", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "topological_sort", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "a_star", to: "graph_algorithm", relation: 0x01 },
    SeedEdge { from: "bfs", to: "dfs", relation: 0x07 },          // BFS ≈ DFS
    SeedEdge { from: "dijkstra", to: "bellman_ford", relation: 0x07 }, // similar
    SeedEdge { from: "kruskal", to: "prim", relation: 0x07 },     // similar (MST)
    SeedEdge { from: "a_star", to: "dijkstra", relation: 0x08 },  // A* ← Dijkstra (derived)

    // DP
    SeedEdge { from: "dynamic_programming", to: "algorithm", relation: 0x02 }, // ⊂
    SeedEdge { from: "memoization", to: "dynamic_programming", relation: 0x05 }, // ∘ compose
    SeedEdge { from: "greedy", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "divide_conquer", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "backtracking", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "recursion", to: "iteration", relation: 0x07 }, // ≈ similar

    // String
    SeedEdge { from: "kmp", to: "string_matching", relation: 0x01 },
    SeedEdge { from: "rabin_karp", to: "string_matching", relation: 0x01 },

    // Numerical
    SeedEdge { from: "newton_method", to: "numerical_method", relation: 0x01 },
    SeedEdge { from: "euler_method", to: "numerical_method", relation: 0x01 },
    SeedEdge { from: "monte_carlo", to: "numerical_method", relation: 0x01 },
    SeedEdge { from: "fft", to: "numerical_method", relation: 0x01 },

    // ML
    SeedEdge { from: "machine_learning", to: "algorithm", relation: 0x02 }, // ML ⊂ algorithm
    SeedEdge { from: "neural_network", to: "machine_learning", relation: 0x01 },
    SeedEdge { from: "gradient_descent", to: "machine_learning", relation: 0x01 },

    // Cross-domain connections
    SeedEdge { from: "sorting", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "searching", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "graph_algorithm", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "string_matching", to: "algorithm", relation: 0x02 },
    SeedEdge { from: "data_structure", to: "algorithm", relation: 0x07 }, // ≈ related
];

pub fn all_nodes() -> Vec<&'static SeedNode> {
    ALGORITHM_NODES.iter().collect()
}
