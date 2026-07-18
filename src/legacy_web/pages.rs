use crate::i18n::Lang;

/// HTML for the Settings window (language preference).
pub fn settings_html(lang: Lang) -> String {
    let t = lang.strings();
    let en_checked = if lang == Lang::En { " checked" } else { "" };
    let zh_checked = if lang == Lang::Zh { " checked" } else { "" };
    // Escape is unnecessary: all strings are static literals we control.
    format!(
        r##"<!DOCTYPE html>
<html lang="{code}">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>
  :root {{
    color-scheme: light dark;
    --bg: #f5f6f8;
    --card: #ffffff;
    --text: #0e1116;
    --muted: #6b7280;
    --accent: #0fb8b0;
    --border: #e5e7eb;
    --hover: #f0fdfa;
  }}
  @media (prefers-color-scheme: dark) {{
    :root {{
      --bg: #121417;
      --card: #1a1c1e;
      --text: #f3f4f6;
      --muted: #9ca3af;
      --border: #2a2e33;
      --hover: #0f2a29;
    }}
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    font: 14px/1.45 -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    padding: 28px 24px;
  }}
  h1 {{
    margin: 0 0 6px;
    font-size: 18px;
    font-weight: 650;
    letter-spacing: -0.02em;
  }}
  .hint {{
    margin: 0 0 20px;
    color: var(--muted);
    font-size: 12.5px;
  }}
  .card {{
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 6px;
  }}
  label.option {{
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 9px;
    cursor: pointer;
    user-select: none;
  }}
  label.option:hover {{ background: var(--hover); }}
  label.option + label.option {{ border-top: 1px solid var(--border); border-radius: 0; }}
  label.option:last-child {{ border-radius: 0 0 9px 9px; }}
  label.option:first-of-type {{ border-radius: 9px 9px 0 0; }}
  input[type="radio"] {{
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
    margin: 0;
  }}
  .label {{ font-weight: 550; }}
</style>
</head>
<body>
  <h1>{language}</h1>
  <p class="hint">{hint}</p>
  <div class="card" role="radiogroup" aria-label="{language}">
    <label class="option">
      <input type="radio" name="lang" value="en"{en_checked}
        onchange="window.ipc.postMessage('lang=en')"/>
      <span class="label">{lang_en}</span>
    </label>
    <label class="option">
      <input type="radio" name="lang" value="zh"{zh_checked}
        onchange="window.ipc.postMessage('lang=zh')"/>
      <span class="label">{lang_zh}</span>
    </label>
  </div>
</body>
</html>
"##,
        code = lang.code(),
        title = t.settings_title,
        language = t.language,
        hint = t.language_hint,
        en_checked = en_checked,
        zh_checked = zh_checked,
        lang_en = t.lang_en,
        lang_zh = t.lang_zh,
    )
}

/// HTML for the zero-config launch page shown while the local kimi daemon is
/// being discovered or started.
///
/// The host drives the status line via `window.__kiminiStatus('<key>')` with
/// keys `starting` / `waiting` / `noKimi` / `invalidUrl` (see
/// `daemon::Status::key`). Manual connect posts `connect=<url>` over IPC; the
/// host enforces loopback-only before navigating.
pub fn launch_html(lang: Lang) -> String {
    let t = lang.strings();
    // Escape is unnecessary: all strings are static literals we control,
    // quote-free by convention (they are embedded in JS string literals).
    format!(
        r##"<!DOCTYPE html>
<html lang="{code}">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Kimini</title>
<style>
  :root {{
    color-scheme: light dark;
    --bg: #f5f6f8;
    --card: #ffffff;
    --text: #0e1116;
    --muted: #6b7280;
    --accent: #0fb8b0;
    --border: #e5e7eb;
  }}
  @media (prefers-color-scheme: dark) {{
    :root {{
      --bg: #121417;
      --card: #1a1c1e;
      --text: #f3f4f6;
      --muted: #9ca3af;
      --border: #2a2e33;
    }}
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    font: 14px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    padding: 0 32px;
    text-align: center;
    user-select: none;
  }}
  .spinner {{
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    animation: spin 0.9s linear infinite;
  }}
  @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
  #status {{ margin: 0; color: var(--muted); max-width: 560px; }}
  pre {{
    display: none;
    margin: 0;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 9px;
    padding: 10px 14px;
    user-select: text;
  }}
  code {{ font: 12.5px ui-monospace, SFMono-Regular, Menlo, monospace; }}
  details {{ color: var(--muted); font-size: 12.5px; }}
  summary {{ cursor: pointer; }}
  .row {{ display: flex; gap: 8px; margin-top: 10px; }}
  input {{
    flex: 1;
    min-width: 320px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--card);
    color: var(--text);
    font: 12.5px ui-monospace, SFMono-Regular, Menlo, monospace;
  }}
  button {{
    padding: 8px 14px;
    border: 0;
    border-radius: 8px;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
    cursor: pointer;
  }}
</style>
</head>
<body>
  <div class="spinner"></div>
  <p id="status">{probing}</p>
  <pre id="install"><code>npm install -g @moonshot-ai/kimi-code</code></pre>
  <details id="manual">
    <summary>{manual}</summary>
    <p>{manual_hint}</p>
    <div class="row">
      <input id="url" type="text" placeholder="http://127.0.0.1:58627/#token=…" spellcheck="false"/>
      <button onclick="kiminiConnect()">{connect}</button>
    </div>
  </details>
<script>
  const M = {{
    starting: "{starting}",
    waiting: "{waiting}",
    noKimi: "{no_kimi}",
    invalidUrl: "{invalid_url}",
  }};
  window.__kiminiStatus = (k) => {{
    if (M[k]) document.getElementById('status').textContent = M[k];
    document.getElementById('install').style.display = k === 'noKimi' ? 'block' : 'none';
    if (k === 'waiting' || k === 'noKimi') document.getElementById('manual').open = true;
  }};
  function kiminiConnect() {{
    const v = document.getElementById('url').value.trim();
    if (v) window.ipc.postMessage('connect=' + v);
  }}
  document.getElementById('url').addEventListener('keydown', (e) => {{
    if (e.key === 'Enter') kiminiConnect();
  }});
</script>
</body>
</html>
"##,
        code = lang.code(),
        probing = t.launch_probing,
        manual = t.launch_manual,
        manual_hint = t.launch_manual_hint,
        connect = t.launch_connect,
        starting = t.launch_starting,
        waiting = t.launch_waiting,
        no_kimi = t.launch_no_kimi,
        invalid_url = t.launch_invalid_url,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_page_contains_both_language_options() {
        let html = settings_html(Lang::En);
        assert!(html.contains("lang=en"));
        assert!(html.contains("lang=zh"));
        assert!(html.contains("English"));
        assert!(html.contains("checked"));
    }

    #[test]
    fn launch_page_wires_status_keys_and_ipc() {
        for lang in [Lang::En, Lang::Zh] {
            let html = launch_html(lang);
            for key in ["starting", "waiting", "noKimi", "invalidUrl"] {
                assert!(html.contains(key), "missing status key {key}");
            }
            assert!(html.contains("__kiminiStatus"));
            assert!(html.contains("window.ipc.postMessage('connect=' + v)"));
        }
    }
}
