// Source: /data/home/swei/claudecode/openclaudecode/src/utils/heatmap.ts
//! Heatmap generation utilities
//!
//! Generates a GitHub-style activity heatmap for the terminal

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// Heatmap options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeatmapOptions {
    /// Terminal width in characters
    pub terminal_width: Option<usize>,
    /// Whether to show month labels
    pub show_month_labels: Option<bool>,
}

/// Daily activity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Number of messages
    pub message_count: usize,
}

/// Percentiles for intensity calculation
#[derive(Debug, Clone)]
struct Percentiles {
    p25: usize,
    p50: usize,
    p75: usize,
}

/// Pre-calculates percentiles from activity data for use in intensity calculations
fn calculate_percentiles(daily_activity: &[DailyActivity]) -> Option<Percentiles> {
    let mut counts: Vec<usize> = daily_activity
        .iter()
        .map(|a| a.message_count)
        .filter(|&c| c > 0)
        .collect();

    if counts.is_empty() {
        return None;
    }

    counts.sort();
    let len = counts.len();

    Some(Percentiles {
        p25: counts[len / 4],
        p50: counts[len / 2],
        p75: counts[(len * 3) / 4],
    })
}

/// Get intensity level (0-4) based on message count and percentiles
fn get_intensity(message_count: usize, percentiles: &Option<Percentiles>) -> usize {
    if message_count == 0 || percentiles.is_none() {
        return 0;
    }

    let p = percentiles.as_ref().unwrap();
    if message_count >= p.p75 {
        return 4;
    }
    if message_count >= p.p50 {
        return 3;
    }
    if message_count >= p.p25 {
        return 2;
    }
    return 1;
}

/// Get heatmap character for intensity level
fn get_heatmap_char(intensity: usize) -> String {
    // Using Unicode block characters with orange color escape code
    let orange = "\x1b[38;2;218;119;86m"; // #da7756
    let reset = "\x1b[0m";

    match intensity {
        0 => format!("{}·{}", orange, reset),
        1 => format!("{}░{}", orange, reset),
        2 => format!("{}▒{}", orange, reset),
        3 => format!("{}▓{}", orange, reset),
        4 => format!("{}█{}", orange, reset),
        _ => format!("{}·{}", orange, reset),
    }
}

/// Converts a DateTime to a date string (YYYY-MM-DD)
fn to_date_string(date: &chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Generates a GitHub-style activity heatmap for the terminal
///
/// # Arguments
/// * `daily_activity` - Array of daily activity data
/// * `options` - Heatmap options
///
/// # Returns
/// String representation of the heatmap
pub fn generate_heatmap(daily_activity: &[DailyActivity], options: HeatmapOptions) -> String {
    let terminal_width = options.terminal_width.unwrap_or(80);
    let show_month_labels = options.show_month_labels.unwrap_or(true);

    // Day labels take 4 characters ("Mon "), calculate weeks that fit
    // Cap at 52 weeks (1 year) to match GitHub style
    let day_label_width = 4;
    let available_width = terminal_width - day_label_width;
    let width = 52.min(10.max(available_width));

    // Build activity map by date
    let mut activity_map = std::collections::HashMap::new();
    for activity in daily_activity {
        activity_map.insert(activity.date.clone(), activity);
    }

    // Pre-calculate percentiles once for all intensity lookups
    let percentiles = calculate_percentiles(daily_activity);

    // Calculate date range - end at today, go back N weeks
    let today = chrono::Local::now().date_naive();

    // Find the Sunday of the current week (start of the week containing today)
    let weekday = today.weekday();
    let days_since_sunday = weekday.num_days_from_sunday() as i64;
    let current_week_start = today - chrono::Duration::days(days_since_sunday);

    // Go back (width - 1) weeks from the current week start
    let start_date = current_week_start - chrono::Duration::weeks((width - 1) as i64);

    // Generate grid (7 rows for days of week, width columns for weeks)
    // Also track which week each month starts for labels
    let mut grid: Vec<Vec<String>> = vec![vec![String::new(); width]; 7];
    let mut month_starts: Vec<(u32, usize)> = vec![];
    let mut last_month: Option<u32> = None;

    let mut current_date = start_date;
    for week in 0..width {
        for day in 0..7 {
            // Don't show future dates
            if current_date > today {
                grid[day][week] = " ".to_string();
                current_date = current_date + chrono::Duration::days(1);
                continue;
            }

            let date_str = to_date_string(&current_date);
            let activity = activity_map.get(&date_str);

            // Track month changes (on day 0 = Sunday of each week)
            if day == 0 {
                let month = current_date.month();
                if last_month != Some(month) {
                    month_starts.push((month, week));
                    last_month = Some(month);
                }
            }

            // Determine intensity level based on message count
            let intensity =
                get_intensity(activity.map(|a| a.message_count).unwrap_or(0), &percentiles);
            grid[day][week] = get_heatmap_char(intensity);

            current_date = current_date + chrono::Duration::days(1);
        }
    }

    // Build output
    let mut lines: Vec<String> = vec![];

    // Month labels - evenly spaced across the grid
    if show_month_labels {
        let month_names = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let unique_months: Vec<u32> = month_starts.iter().map(|(m, _)| *m).collect();
        let label_width = width / unique_months.len().max(1);

        let month_labels: String = unique_months
            .iter()
            .filter_map(|&m| month_names.get((m - 1) as usize).copied())
            .map(|s| {
                // Pad the string to label_width
                let mut result = s.to_string();
                while result.len() < label_width {
                    result.push(' ');
                }
                result
            })
            .collect();

        lines.push(format!("    {}", month_labels));
    }

    // Day labels
    let day_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    // Grid
    for day in 0..7 {
        // Only show labels for Mon, Wed, Fri
        let label = if [1, 3, 5].contains(&day) {
            let mut s = day_labels[day].to_string();
            while s.len() < 3 {
                s.push(' ');
            }
            s
        } else {
            "   ".to_string()
        };
        let row = format!("{} {}", label, grid[day].join(""));
        lines.push(row);
    }

    // Legend
    lines.push(String::new());
    let orange = "\x1b[38;2;218;119;86m";
    let reset = "\x1b[0m";
    lines.push(format!(
        "    Less {}░{} {}▒{} {}▓{} {}█{} More",
        orange, reset, orange, reset, orange, reset, orange, reset
    ));

    lines.join("\n")
}
