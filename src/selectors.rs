use lazy_static::lazy_static;
use scraper::Selector;

lazy_static! {
    pub static ref PRINTUSER: Selector = Selector::parse("span.printuser").unwrap();
    pub static ref VOTE: Selector = Selector::parse("span[style^='color']").unwrap();
    pub static ref PAGESOURCE: Selector = Selector::parse("div.page-source").unwrap();
    pub static ref TR: Selector = Selector::parse("tr").unwrap();
    pub static ref TD: Selector = Selector::parse("td").unwrap();
    pub static ref ODATE: Selector = Selector::parse("span.odate").unwrap();
    pub static ref A: Selector = Selector::parse("a").unwrap();
    pub static ref PAGE: Selector = Selector::parse("div.page").unwrap();
    pub static ref STAR: Selector = Selector::parse("span.rating span.page-rate-list-pages-start").unwrap();
    pub static ref SET: Selector = Selector::parse("span.set").unwrap();
    pub static ref KEY: Selector = Selector::parse("span.name").unwrap();
    pub static ref VALUE: Selector = Selector::parse("span.value").unwrap();
    pub static ref PAGERNO: Selector = Selector::parse("span.pager-no").unwrap();
    pub static ref H1: Selector = Selector::parse("h1").unwrap();
    pub static ref IMG: Selector = Selector::parse("img").unwrap();
}