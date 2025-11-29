use std::collections::HashMap;

pub struct VotingMechanism {
    k: usize, // Threshold k for first-to-ahead-by-k
}

impl VotingMechanism {
    pub fn new(k: usize) -> Self {
        Self { k }
    }

    // Algorithm 2: First-to-ahead-by-k voting
    pub fn vote(&self, candidates: Vec<String>) -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        
        for candidate in candidates {
            *counts.entry(candidate.clone()).or_insert(0) += 1;
            
            // Check if any candidate is ahead by k
            let current_count = counts[&candidate];
            
            // Find max of others
            let mut max_other = 0;
            for (other_cand, &other_count) in &counts {
                if other_cand != &candidate {
                    max_other = std::cmp::max(max_other, other_count);
                }
            }

            if current_count >= max_other + self.k {
                return Some(candidate);
            }
        }
        
        // If no one wins by k margin, return the one with max votes (fallback)
        counts.into_iter().max_by_key(|&(_, count)| count).map(|(cand, _)| cand)
    }
}



