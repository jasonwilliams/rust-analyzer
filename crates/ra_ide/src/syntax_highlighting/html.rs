//! Renders a bit of code as HTML.

use ra_db::SourceDatabase;
use ra_syntax::AstNode;

use crate::{FileId, HighlightedRange, RootDatabase};

use super::highlight;

pub(crate) fn highlight_as_html(db: &RootDatabase, file_id: FileId, rainbow: bool) -> String {
    let parse = db.parse(file_id);

    fn rainbowify(seed: u64) -> String {
        use rand::prelude::*;
        let mut rng = SmallRng::seed_from_u64(seed);
        format!(
            "hsl({h},{s}%,{l}%)",
            h = rng.gen_range::<u16, _, _>(0, 361),
            s = rng.gen_range::<u16, _, _>(42, 99),
            l = rng.gen_range::<u16, _, _>(40, 91),
        )
    }

    let mut ranges = highlight(db, file_id, None);
    ranges.sort_by_key(|it| it.range.start());
    // quick non-optimal heuristic to intersect token ranges and highlighted ranges
    let mut frontier = 0;
    let mut could_intersect: Vec<&HighlightedRange> = Vec::new();

    let mut buf = String::new();
    buf.push_str(&STYLE);
    buf.push_str("<pre><code>");
    let tokens = parse.tree().syntax().descendants_with_tokens().filter_map(|it| it.into_token());
    for token in tokens {
        could_intersect.retain(|it| token.text_range().start() <= it.range.end());
        while let Some(r) = ranges.get(frontier) {
            if r.range.start() <= token.text_range().end() {
                could_intersect.push(r);
                frontier += 1;
            } else {
                break;
            }
        }
        let text = html_escape(&token.text());
        let ranges = could_intersect
            .iter()
            .filter(|it| token.text_range().is_subrange(&it.range))
            .collect::<Vec<_>>();
        if ranges.is_empty() {
            buf.push_str(&text);
        } else {
            let classes = ranges
                .iter()
                .map(|it| it.highlight.to_string().replace('.', " "))
                .collect::<Vec<_>>()
                .join(" ");
            let binding_hash = ranges.first().and_then(|x| x.binding_hash);
            let color = match (rainbow, binding_hash) {
                (true, Some(hash)) => format!(
                    " data-binding-hash=\"{}\" style=\"color: {};\"",
                    hash,
                    rainbowify(hash)
                ),
                _ => "".into(),
            };
            buf.push_str(&format!("<span class=\"{}\"{}>{}</span>", classes, color, text));
        }
    }
    buf.push_str("</code></pre>");
    buf
}

//FIXME: like, real html escaping
fn html_escape(text: &str) -> String {
    text.replace("<", "&lt;").replace(">", "&gt;")
}

const STYLE: &str = "
<style>
body                { margin: 0; }
pre                 { color: #DCDCCC; background: #3F3F3F; font-size: 22px; padding: 0.4em; }

.comment            { color: #7F9F7F; }
.string             { color: #CC9393; }
.field              { color: #94BFF3; }
.function           { color: #93E0E3; }
.parameter          { color: #94BFF3; }
.text               { color: #DCDCCC; }
.type               { color: #7CB8BB; }
.type.builtin       { color: #8CD0D3; }
.type.param         { color: #20999D; }
.attribute          { color: #94BFF3; }
.literal            { color: #BFEBBF; }
.literal.numeric    { color: #6A8759; }
.macro              { color: #94BFF3; }
.module             { color: #AFD8AF; }
.variable           { color: #DCDCCC; }
.variable.mut       { color: #DCDCCC; text-decoration: underline; }

.keyword            { color: #F0DFAF; }
.keyword.unsafe     { color: #DFAF8F; }
.keyword.control    { color: #F0DFAF; font-weight: bold; }
</style>
";
