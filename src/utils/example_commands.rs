use std::collections::HashMap;

pub fn count_and_sort_items(items: Vec<String>, top_n: usize) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for item in items {
        *counts.entry(item).or_insert(0) += 1;
    }

    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .iter()
        .take(top_n)
        .map(|(item, count)| format!("{:6} {}", count, item))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn pick_diverse_core_files(sorted_paths: Vec<String>, want: usize) -> Vec<String> {
    let non_core_patterns = [
        "package-lock.json",
        "yarn.lock",
        "bun.lock",
        "pnpm-lock.yaml",
        "dist",
        "build",
        "node_modules",
        ".min.js",
        ".min.css",
        ".json",
        ".yaml",
        ".yml",
        ".md",
    ];

    let mut picked = Vec::new();
    let mut seen_basenames = std::collections::HashSet::new();
    let mut dir_tally: HashMap<String, usize> = HashMap::new();

    for cap in 1..=want {
        if picked.len() >= want {
            break;
        }
        for p in &sorted_paths {
            if picked.len() >= want {
                break;
            }

            let is_non_core = non_core_patterns
                .iter()
                .any(|pattern| p.contains(pattern) || p.ends_with(pattern));
            if is_non_core {
                continue;
            }

            let base = std::path::Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if base.is_empty() || seen_basenames.contains(&base) {
                continue;
            }

            let dir = std::path::Path::new(p)
                .parent()
                .map(|d| d.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());

            let current_count = *dir_tally.get(&dir).unwrap_or(&0);
            if current_count >= cap {
                continue;
            }

            picked.push(base.clone());
            seen_basenames.insert(base);
            *dir_tally.entry(dir).or_insert(0) += 1;
        }
    }

    if picked.len() >= want {
        picked
    } else {
        vec![]
    }
}
