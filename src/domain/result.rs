use serde_json::Value;

#[allow(unused)]
pub struct NonEmptyResults(Vec<Value>);

#[allow(unused)]
impl NonEmptyResults {
    pub fn results(&self) -> &[Value] {
        &self.0
    }
}

#[allow(unused)]
pub enum QueryResults {
    Empty,
    NonEmpty(NonEmptyResults),
}

#[allow(unused)]
impl QueryResults {
    pub fn new(results: Vec<Value>) -> Self {
        if results.is_empty() {
            return QueryResults::Empty;
        }

        QueryResults::NonEmpty(NonEmptyResults(results))
    }
}
