use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UrlMetadata {
    pub title: String,
    pub description: String,
    pub favicon_url: String,
}

#[tauri::command]
pub async fn fetch_url_metadata(url: String) -> Result<UrlMetadata, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; VaultKeeper/1.0)")
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    let final_url = response.url().clone();
    let scheme = final_url.scheme();
    let host = final_url.host_str().unwrap_or("");
    let origin = format!("{}://{}", scheme, host);

    let html = response.text().await.map_err(|e| e.to_string())?;
    let html_lower = html.to_lowercase();

    let title = extract_title(&html, &html_lower);
    let description = extract_meta_content(&html, &html_lower, "description");
    let favicon_url = extract_favicon(&html, &html_lower, &origin)
        .unwrap_or_else(|| format!("{}/favicon.ico", origin));

    Ok(UrlMetadata { title, description, favicon_url })
}

fn extract_title(html: &str, lower: &str) -> String {
    let start = match lower.find("<title") {
        Some(i) => i,
        None => return String::new(),
    };
    let tag_end = match lower[start..].find('>') {
        Some(i) => start + i + 1,
        None => return String::new(),
    };
    let end = match lower[tag_end..].find("</title>") {
        Some(i) => tag_end + i,
        None => return String::new(),
    };
    html_decode(html[tag_end..end].trim())
}

fn extract_meta_content(html: &str, lower: &str, name: &str) -> String {
    let name_dq = format!("name=\"{}\"", name);
    let name_sq = format!("name='{}'", name);

    let mut pos = 0;
    while let Some(rel_pos) = lower[pos..].find("<meta") {
        let meta_start = pos + rel_pos;
        let tag_end = lower[meta_start..].find('>').map(|i| meta_start + i + 1)
            .unwrap_or(lower.len());
        let tag_lower = &lower[meta_start..tag_end];
        let tag_orig = &html[meta_start..tag_end.min(html.len())];

        if tag_lower.contains(name_dq.as_str()) || tag_lower.contains(name_sq.as_str()) {
            if let Some(val) = extract_attr(tag_orig, tag_lower, "content") {
                return html_decode(&val);
            }
        }
        pos = meta_start + 1;
    }
    String::new()
}

fn extract_favicon(html: &str, lower: &str, origin: &str) -> Option<String> {
    let mut pos = 0;
    while let Some(rel_pos) = lower[pos..].find("<link") {
        let link_start = pos + rel_pos;
        let tag_end = lower[link_start..].find('>').map(|i| link_start + i + 1)
            .unwrap_or(lower.len());
        let tag_lower = &lower[link_start..tag_end];
        let tag_orig = &html[link_start..tag_end.min(html.len())];

        let is_icon = tag_lower.contains("rel=\"icon\"")
            || tag_lower.contains("rel='icon'")
            || tag_lower.contains("rel=\"shortcut icon\"")
            || tag_lower.contains("rel='shortcut icon'")
            || tag_lower.contains("icon\"")  // handles rel="apple-touch-icon" etc
        ;

        if is_icon {
            if let Some(href) = extract_attr(tag_orig, tag_lower, "href") {
                let resolved = resolve_url(&href, origin);
                if !resolved.is_empty() {
                    return Some(resolved);
                }
            }
        }
        pos = link_start + 1;
    }
    None
}

/// Extract the value of `attr` from an HTML tag snippet.
/// `tag_lower` is the lowercased version of `tag_orig`.
fn extract_attr(tag_orig: &str, tag_lower: &str, attr: &str) -> Option<String> {
    // Try double-quote, single-quote, unquoted
    for (pattern, end_char) in [
        (format!("{}=\"", attr), '"'),
        (format!("{}='", attr), '\''),
    ] {
        if let Some(start) = tag_lower.find(pattern.as_str()) {
            let val_start = start + pattern.len();
            if val_start > tag_orig.len() { continue; }
            if let Some(end) = tag_orig[val_start..].find(end_char) {
                return Some(tag_orig[val_start..val_start + end].to_string());
            }
        }
    }
    None
}

fn resolve_url(href: &str, origin: &str) -> String {
    let href = href.trim();
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{}", href)
    } else if href.starts_with('/') {
        format!("{}{}", origin, href)
    } else if href.is_empty() {
        String::new()
    } else {
        format!("{}/{}", origin, href)
    }
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}
