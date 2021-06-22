use tokio;
use kuchiki;
use kuchiki::traits::TendrilSink;

use rp_watch::shorelandsearch::ShoreLandSearchForm;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let form = ShoreLandSearchForm::new(Some(2500), None, Some(2), Some(2), 2021, 9, 6);
    let resp = reqwest::Client::new()
        .post("https://macapartments.secure.force.com/mac/ShorelandSiteSearch")
        .form(&form)
        .send()
        .await?;
    let html = resp.text().await?;
    let document = kuchiki::parse_html().one(html);
    for apartment in document.select(r"form#shorelandsearch\:searchresultspanelform div.content-wrapper.clearfix").unwrap() {
        let apartment = apartment.as_node();
        let room = apartment
            .select_first("a.caps")
            .unwrap()
            .as_node()
            .first_child()
            .unwrap()
            .as_text()
            .unwrap()
            .borrow()
            .trim()
            .to_string();
        dbg!(room);
        for info in apartment.select("table.results-table > tbody > tr > td").unwrap() {
            let info = info
                .as_node()
                .first_child()
                .unwrap()
                .as_text()
                .unwrap()
                .borrow()
                .trim()
                .to_string();
            dbg!(info);
        }
    }

    Ok(())
}
