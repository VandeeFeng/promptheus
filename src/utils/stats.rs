use crate::core::data::PromptStats;
use crate::utils::output::OutputStyle;

/// Utilities for calculating and displaying prompt statistics
pub struct StatsCalculator;

impl StatsCalculator {
    /// Print formatted prompt statistics
    pub fn print_stats(stats: &PromptStats) {
        OutputStyle::print_header("📊 Prompt Statistics");

        OutputStyle::print_field_colored("Total prompts", &stats.total_prompts.to_string(), OutputStyle::info);
        OutputStyle::print_field_colored("Total tags", &stats.total_tags.to_string(), OutputStyle::info);
        OutputStyle::print_field_colored("Categories used", &stats.total_categories.to_string(), OutputStyle::info);

        if !stats.tag_counts.is_empty() {
            println!("\n🏷️  {}:", OutputStyle::header("Most used tags"));
            let mut sorted_tags: Vec<_> = stats.tag_counts.iter().collect();
            sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

            for (tag, count) in sorted_tags.iter().take(10) {
                println!("  {}: {}", OutputStyle::tags(tag), OutputStyle::info(&count.to_string()));
            }
        }

        if !stats.category_counts.is_empty() {
            println!("\n📁 {}:", OutputStyle::header("Categories"));
            let mut sorted_categories: Vec<_> = stats.category_counts.iter().collect();
            sorted_categories.sort_by(|a, b| b.1.cmp(a.1));

            for (category, count) in sorted_categories {
                println!("  {}: {}", OutputStyle::tag(category), OutputStyle::info(&count.to_string()));
            }
        }
    }
}