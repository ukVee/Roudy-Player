use crate::types::QueryParams;


fn get_param<'a>(param: &str, url: &'a str) -> Option<&'a str> {
    let prefix = format!("{param}=");
    let start_of_param = url.find(&prefix)? + prefix.len();
    let rest_of_url = &url[start_of_param..];

    match rest_of_url.find('&') {
        Some(start_of_new_param) => {
            Some(&rest_of_url[..start_of_new_param])
        }
        None => Some(rest_of_url)
    }
}

pub fn parse_query_params(url: String) -> QueryParams {
    QueryParams {
        authorization_code: get_param("code", &url).map(String::from),
        csrf_state: get_param("state", &url).map(String::from)
    }
}