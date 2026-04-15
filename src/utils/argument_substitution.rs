pub fn parse_arguments(args: &str) -> Vec<String> {
    if args.trim().is_empty() {
        return vec![];
    }
    args.split_whitespace().map(String::from).collect()
}

pub fn parse_argument_names(argument_names: Option<&str>) -> Vec<String> {
    match argument_names {
        Some(names) if !names.is_empty() => names
            .split_whitespace()
            .filter(|name| !name.trim().is_empty() && !name.chars().all(|c| c.is_ascii_digit()))
            .map(String::from)
            .collect(),
        _ => vec![],
    }
}

pub fn generate_progressive_argument_hint(
    arg_names: &[String],
    typed_args: &[String],
) -> Option<String> {
    let remaining: Vec<&String> = arg_names.iter().skip(typed_args.len()).collect();
    if remaining.is_empty() {
        return None;
    }
    Some(
        remaining
            .iter()
            .map(|s| format!("[{}]", s))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub fn substitute_arguments(
    content: &str,
    args: Option<&str>,
    append_if_no_placeholder: bool,
) -> String {
    let args = match args {
        Some(a) => a,
        None => return content.to_string(),
    };

    let parsed_args = parse_arguments(args);
    let mut result = content.to_string();

    for (i, _) in parsed_args.iter().enumerate() {
        let placeholder = format!("$ARGUMENTS[{}]", i);
        result = result.replace(
            &placeholder,
            parsed_args.get(i).map(|s| s.as_str()).unwrap_or(""),
        );
    }

    result = substitute_digit_shorthand(&result, &parsed_args);
    result = result.replace("$ARGUMENTS", args);

    if result == content && append_if_no_placeholder && !args.is_empty() {
        result = format!("{}\n\nARGUMENTS: {}", content, args);
    }

    result
}

fn substitute_digit_shorthand(content: &str, parsed_args: &[String]) -> String {
    let mut result = content.to_string();
    let mut offset = 0;

    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let mut j = i + 1;
            let mut digits = String::new();
            while j < chars.len() && chars[j].is_ascii_digit() {
                digits.push(chars[j]);
                j += 1;
            }
            if !digits.is_empty() && (j >= chars.len() || !chars[j].is_alphanumeric()) {
                if let Ok(idx) = digits.parse::<usize>() {
                    if let Some(arg) = parsed_args.get(idx) {
                        result = format!("{}{}{}", &result[..i], arg, &result[j..]);
                        i += arg.len();
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    result
}
