//! Information-Theoretic Entropy (Claude Shannon, 1948), Yule's Characteristic $K$ (1944),
//! Simpson's Diversity Index, Honoré's Hapax Statistic $R$, and Zipf's Law Exponent Fitting.

#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyRankEntry {
    pub rank: usize,
    pub lemma: String,
    pub count: usize,
    pub relative_freq: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZipfDistributionFit {
    /// Estimated Zipf slope $\alpha$ in $f(r) \propto r^{-\alpha}$ (typically $\approx 1.0$).
    pub slope_alpha: f32,
    pub top_ranks: Vec<FrequencyRankEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntropyAndRichnessReport {
    /// Shannon character entropy $H_{\text{char}}$ in bits per character (typically 4.0..=4.8 for Russian/English).
    pub char_entropy_bits: f32,
    /// Shannon word-level entropy $H_{\text{word}}$ in bits per token.
    pub word_entropy_bits: f32,
    /// Normalized redundancy $R = 1 - H / H_{\max}$.
    pub structural_redundancy: f32,
    /// Total word tokens $N$.
    pub total_tokens: usize,
    /// Unique vocabulary types $V$.
    pub unique_types: usize,
    /// Type-Token Ratio $V / N$.
    pub type_token_ratio: f32,
    /// Hapax legomena $V_1$ (words appearing exactly once).
    pub hapax_legomena_count: usize,
    /// Dis legomena $V_2$ (words appearing exactly twice).
    pub dis_legomena_count: usize,
    /// Yule's Characteristic $K = 10^4 \cdot \frac{\sum i^2 V_i - N}{N^2}$.
    /// Lower $K$ (50–100) indicates rich, varied vocabulary; higher $K$ (>150) indicates repetitive vocabulary.
    pub yules_characteristic_k: f32,
    /// Honoré's Statistic $R = \frac{100 \ln N}{1 - V_1 / V}$. Higher values indicate richer lexical creativity.
    pub honores_statistic_r: f32,
    /// Simpson's Diversity Index $D = 1 - \frac{\sum n_i (n_i - 1)}{N (N - 1)}$.
    pub simpsons_diversity: f32,
    pub zipf_fit: ZipfDistributionFit,
}

/// Computes full information-theoretic entropy and lexical richness metrics in $O(N \log V)$ time
/// without external hash-table dependencies (using sorted deterministic vectors).
pub fn compute_lexical_entropy(text: &str) -> EntropyAndRichnessReport {
    // 1. Character-level Shannon entropy
    let mut chars: Vec<char> = text
        .chars()
        .filter(|c| !c.is_control())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let total_chars = chars.len();
    chars.sort_unstable();

    let mut char_entropy = 0.0f32;
    let mut distinct_chars = 0usize;
    if total_chars > 0 {
        let inv_total = 1.0 / (total_chars as f32);
        let mut i = 0;
        while i < total_chars {
            let current = chars[i];
            let mut run = 1usize;
            while i + run < total_chars && chars[i + run] == current {
                run += 1;
            }
            distinct_chars += 1;
            let p = (run as f32) * inv_total;
            char_entropy -= p * p.log2();
            i += run;
        }
    }

    let max_char_entropy = if distinct_chars > 1 {
        (distinct_chars as f32).log2()
    } else {
        1.0
    };
    let structural_redundancy = (1.0 - (char_entropy / max_char_entropy)).clamp(0.0, 1.0);

    // 2. Word-level tokenization and frequency spectrum
    let mut words: Vec<String> = text
        .split_whitespace()
        .map(|tok| {
            tok.trim_matches(|c: char| !c.is_alphabetic() && c != '-')
                .chars()
                .flat_map(|c| c.to_lowercase())
                .collect::<String>()
        })
        .filter(|w| !w.is_empty())
        .collect();

    let n_tokens = words.len();
    if n_tokens == 0 {
        return EntropyAndRichnessReport {
            char_entropy_bits: char_entropy,
            word_entropy_bits: 0.0,
            structural_redundancy,
            total_tokens: 0,
            unique_types: 0,
            type_token_ratio: 0.0,
            hapax_legomena_count: 0,
            dis_legomena_count: 0,
            yules_characteristic_k: 0.0,
            honores_statistic_r: 0.0,
            simpsons_diversity: 0.0,
            zipf_fit: ZipfDistributionFit {
                slope_alpha: 1.0,
                top_ranks: Vec::new(),
            },
        };
    }

    words.sort_unstable();

    let mut type_counts: Vec<(String, usize)> = Vec::new();
    let mut idx = 0;
    while idx < n_tokens {
        let current = &words[idx];
        let mut count = 1usize;
        while idx + count < n_tokens && &words[idx + count] == current {
            count += 1;
        }
        type_counts.push((current.clone(), count));
        idx += count;
    }

    // Sort descending by frequency, then alphabetically
    type_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let v_types = type_counts.len();
    let mut hapax = 0usize;
    let mut dis = 0usize;
    let mut word_entropy = 0.0f32;
    let mut sum_i_sq_vi = 0.0f64;
    let mut simpson_num = 0.0f64;

    let inv_n = 1.0 / (n_tokens as f32);
    for (_, count) in &type_counts {
        let c = *count;
        if c == 1 {
            hapax += 1;
        } else if c == 2 {
            dis += 1;
        }
        let p = (c as f32) * inv_n;
        word_entropy -= p * p.log2();

        let cf = c as f64;
        sum_i_sq_vi += cf * cf;
        simpson_num += cf * (cf - 1.0);
    }

    let nf = n_tokens as f64;
    let yules_k = if n_tokens > 1 {
        (10_000.0 * (sum_i_sq_vi - nf) / (nf * nf)).max(0.0) as f32
    } else {
        0.0
    };

    let hapax_ratio = (hapax as f32) / (v_types.max(1) as f32);
    let denom_honore = (1.0 - hapax_ratio).max(0.05);
    let honores_r = (100.0 * (n_tokens as f32).ln()) / denom_honore;

    let simpsons_d = if n_tokens > 1 {
        (1.0 - (simpson_num / (nf * (nf - 1.0)))).clamp(0.0, 1.0) as f32
    } else {
        1.0
    };

    // Estimate Zipf slope alpha from top rank vs median rank
    let top_freq = type_counts[0].1 as f32;
    let last_freq = type_counts[type_counts.len() - 1].1 as f32;
    let slope_alpha = if v_types > 2 && top_freq > last_freq {
        ((top_freq / last_freq).ln() / (v_types as f32).ln()).clamp(0.2, 2.0)
    } else {
        0.95
    };

    let top_ranks = type_counts
        .into_iter()
        .take(8)
        .enumerate()
        .map(|(r, (lemma, count))| FrequencyRankEntry {
            rank: r + 1,
            lemma,
            count,
            relative_freq: (count as f32) / (n_tokens as f32),
        })
        .collect();

    EntropyAndRichnessReport {
        char_entropy_bits: char_entropy,
        word_entropy_bits: word_entropy,
        structural_redundancy,
        total_tokens: n_tokens,
        unique_types: v_types,
        type_token_ratio: (v_types as f32) / (n_tokens as f32),
        hapax_legomena_count: hapax,
        dis_legomena_count: dis,
        yules_characteristic_k: yules_k,
        honores_statistic_r: honores_r,
        simpsons_diversity: simpsons_d,
        zipf_fit: ZipfDistributionFit {
            slope_alpha,
            top_ranks,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_and_yules_k() {
        let sample = "Тишина имеет собственный вес. Когда взгляд скользит по широкому полю бумаги, каждая засечка удерживает равновесие.";
        let rep = compute_lexical_entropy(sample);
        assert!(rep.char_entropy_bits > 3.5);
        assert!(rep.word_entropy_bits > 3.0);
        assert!(rep.unique_types >= 14);
        assert!(rep.simpsons_diversity > 0.85);
    }
}
