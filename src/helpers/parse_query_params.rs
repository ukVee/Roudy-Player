

pub fn parse_query_params(mut url: String) -> Option<String> {
    let index = url.find("code=");
    let mut parsed_param: Option<String> = None;
    match index {
        Some(i) => {
            let second_halve = url.split_off(i);
            if let Some(end_of_code) = second_halve.find("&") {
                parsed_param = Some(second_halve[0..end_of_code].to_string());
            }
            
        }
        None => {
            panic!("Failed to find code param!");
        }
    }
    parsed_param
}