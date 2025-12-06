use std::collections::HashMap;

pub struct VotingMechanism {
    k: usize, // threshold k for first-to-ahead-by-k
}

impl VotingMechanism {
    pub fn new(k: usize) -> Self {
        Self { k }
    }

    pub fn vote(&self, candidates: Vec<String>) -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        
        for candidate in candidates {
            *counts.entry(candidate.clone()).or_insert(0) += 1;
            
            let current_count = counts[&candidate];
            
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
        
        counts.into_iter().max_by_key(|&(_, count)| count).map(|(cand, _)| cand)
    }
}



