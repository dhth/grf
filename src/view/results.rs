use serde_json::Value;
use tabled::builder::Builder;
use tabled::settings::style::Style;

pub fn get_results(results: &Value) -> Option<String> {
    let results_array = match results {
        Value::Array(arr) => Some(arr),
        Value::Object(obj) => match obj.get("results") {
            Some(Value::Array(arr)) => Some(arr),
            _ => None,
        },
        _ => None,
    };

    let results_array = results_array?;

    if results_array.is_empty() {
        return None;
    }

    let mut builder = Builder::default();

    if let Some(Value::Object(first)) = results_array.first() {
        let headers: Vec<String> = first.keys().cloned().collect();
        builder.push_record(&headers);

        for result in results_array {
            if let Value::Object(row) = result {
                let cells: Vec<String> = headers
                    .iter()
                    .map(|h| {
                        row.get(h)
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Null => "null".to_string(),
                                _ => v.to_string(),
                            })
                            .unwrap_or_else(|| "".to_string())
                    })
                    .collect();
                builder.push_record(cells);
            }
        }
    }

    let mut table = builder.build();

    table.with(Style::psql());

    Some(table.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn get_results_returns_correct_table_for_array_of_objects() {
        // GIVEN
        let value = serde_json::json!([
            {"language": "Rust", "creator": "Graydon Hoare", "year": 2010},
            {"language": "Python", "creator": "Guido van Rossum", "year": 1991},
            {"language": "Go", "creator": "Rob Pike", "year": 2009}
        ]);

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         creator          | language | year 
        ------------------+----------+------
         Graydon Hoare    | Rust     | 2010 
         Guido van Rossum | Python   | 1991 
         Rob Pike         | Go       | 2009
        ");
    }

    #[test]
    fn get_results_handles_results_in_object() {
        // GIVEN
        let value = serde_json::json!({
            "results": [
                {"language": "Rust", "year": 2010},
                {"language": "Python", "year": 1991}
            ]
        });

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         language | year 
        ----------+------
         Rust     | 2010 
         Python   | 1991
        ");
    }

    #[test]
    fn get_results_formats_null_values_correctly() {
        // GIVEN
        let value = serde_json::json!([
            {"language": "Rust", "creator": null},
            {"language": "Python", "creator": "Guido van Rossum"}
        ]);

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         creator          | language 
        ------------------+----------
         null             | Rust     
         Guido van Rossum | Python
        ");
    }

    #[test]
    fn get_results_converts_non_string_values_to_string() {
        // GIVEN
        let value = serde_json::json!([
            {"version": "1.0", "stable": true, "downloads": 1000},
            {"version": "2.0", "stable": false, "downloads": 5000}
        ]);

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         downloads | stable | version 
        -----------+--------+---------
         1000      | true   | 1.0     
         5000      | false  | 2.0
        ");
    }

    #[test]
    fn get_results_skips_non_object_array_elements() {
        // GIVEN
        let value = serde_json::json!([
            {"language": "Rust", "creator": "Graydon Hoare"},
            "invalid",
            {"language": "Python", "creator": "Guido van Rossum"}
        ]);

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         creator          | language 
        ------------------+----------
         Graydon Hoare    | Rust     
         Guido van Rossum | Python
        ");
    }

    #[test]
    fn get_results_shows_empty_string_for_missing_columns() {
        // GIVEN
        let value = serde_json::json!([
            {"language": "Rust", "creator": "Graydon Hoare", "year": 2010},
            {"language": "Python", "creator": "Guido van Rossum"},
            {"language": "Go", "year": 2009}
        ]);

        // WHEN
        let result = get_results(&value).expect("result should've been Some");

        // THEN
        assert_snapshot!(result, @r"
         creator          | language | year 
        ------------------+----------+------
         Graydon Hoare    | Rust     | 2010 
         Guido van Rossum | Python   |      
                          | Go       | 2009
        ");
    }

    #[test]
    fn get_results_returns_none_for_non_array_non_object_input() {
        // GIVEN
        let value = serde_json::json!("just a string");

        // WHEN
        let result = get_results(&value);

        // THEN
        assert!(result.is_none());
    }

    #[test]
    fn get_results_returns_none_for_empty_array() {
        // GIVEN
        let value = serde_json::json!([]);

        // WHEN
        let result = get_results(&value);

        // THEN
        assert!(result.is_none());
    }
}
