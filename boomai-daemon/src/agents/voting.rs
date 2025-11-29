use std::collections::HashMap;

pub struct VotingMechanism {
    k: usize, // Threshold k for first-to-ahead-by-k
}

impl VotingMechanism {
    pub fn new(k: usize) -> Self {
        Self { k }
    }

    // Algorithm 2: First-to-ahead-by-k voting with normalization
    pub fn vote(&self, candidates: Vec<String>) -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        
        // Normalize candidates for voting buckets
        let normalized_candidates: Vec<(String, String)> = candidates.iter()
            .map(|c| (self.normalize(c), c.clone()))
            .collect();

        for (norm, original) in &normalized_candidates {
            *counts.entry(norm.clone()).or_insert(0) += 1;
            
            // Check if any candidate is ahead by k
            let current_count = counts[norm];
            
            // Find max of others
            let mut max_other = 0;
            for (other_norm, &other_count) in &counts {
                if other_norm != norm {
                    max_other = std::cmp::max(max_other, other_count);
                }
            }

            if current_count >= max_other + self.k {
                return Some(original.clone()); // Return the original text of the winner
            }
        }
        
        // If no one wins by k margin, return the one with max votes (fallback)
        counts.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(winner_norm, _)| {
                // Find the original text for this normalized winner
                normalized_candidates.iter()
                    .find(|(n, _)| n == &winner_norm)
                    .map(|(_, o)| o.clone())
                    .unwrap_or(winner_norm)
            })
    }

    fn normalize(&self, text: &str) -> String {
        // Basic normalization: trim, lowercase, remove punctuation at end
        let mut s = text.trim().to_lowercase();
        if s.ends_with('.') {
            s.pop();
        }
        s
    }
}



