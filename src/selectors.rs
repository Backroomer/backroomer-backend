use lazy_static::lazy_static;
use scraper::Selector;

macro_rules! build_selector {
    ($name: ident, $selector: expr) => {
        lazy_static! {
            pub static ref $name: Selector = Selector::parse($selector).unwrap();
        }
    };
}

build_selector!(PRINTUSER, "span.printuser");
build_selector!(VOTE, "span[style^='color']");
build_selector!(PAGESOURCE, "div.page-source");
build_selector!(TR, "tr");
build_selector!(TD, "td");
build_selector!(ODATE, "span.odate");
build_selector!(A, "a");
build_selector!(PAGE, "div.page");
build_selector!(STAR, "span.rating span.page-rate-list-pages-start");
build_selector!(SET, "span.set");
build_selector!(KEY, "span.name");
build_selector!(VALUE, "span.value");
build_selector!(PAGERNO, "span.pager-no");
build_selector!(H1, "h1");
build_selector!(IMG, "img");