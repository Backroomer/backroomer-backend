use lazy_static::lazy_static;
use scraper::Selector;

macro_rules! build_selectors {
    ($(($name:ident, $selector:expr)),* $(,)?) => {
        lazy_static! {
            $(
                pub static ref $name: Selector = Selector::parse($selector).unwrap();
            )*
        }
    };
}

build_selectors!(
    (A, "a"),
    (H1, "h1"),
    (IMG, "img"),
    (TR, "tr"),
    (TD, "td"),
    (PAGE, "div.page"),
    (PAGESOURCE, "div.page-source"),
    (PAGERNO, "span.pager-no"),
    (ALTER, "div#page-content li"),
    (PRINTUSER, "span.printuser"),
    (VOTE, "span[style^='color']"),
    (STAR, "span.rating span.page-rate-list-pages-start"),
    (ODATE, "span.odate"),
    (SET, "span.set"),
    (KEY, "span.name"),
    (VALUE, "span.value"),
    (TABLE, "table.wiki-content-table"),
);