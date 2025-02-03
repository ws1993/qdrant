use crate::issue::Issue;
use crate::solution::Solution;

pub struct TooManyCollections;

impl Issue for TooManyCollections {
    fn instance_id(&self) -> &str {
        "" // Only one issue for the whole app
    }

    fn name() -> &'static str {
        "TOO_MANY_COLLECTIONS"
    }

    fn related_collection(&self) -> Option<String> {
        None
    }

    fn description(&self) -> String {
        "It looks like you have too many collections.\nIf your architecture creates collections programmatically, it's probably better to restructure your solution into a fixed number of them. \nLearn more here: https://qdrant.tech/documentation/guides/multiple-partitions/".to_string()
    }

    fn solution(&self) -> Solution {
        Solution::Refactor(
            "Restructure your solution into a fixed number of collections".to_string(),
        )
    }
}
