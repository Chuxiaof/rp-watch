use futures::{future, stream, StreamExt};
use kuchiki;
use kuchiki::traits::TendrilSink;
use regex::Regex;
use tokio;

use rp_watch::shorelandsearch::ShoreLandSearchForm;

static RENT_LOWER: u32 = 1700;
static RENT_UPPER: u32 = 2500;
static REND_STEP: u32 = 100;
static CONCURRENT_REQUESTS: usize = 8;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let forms = (RENT_LOWER..RENT_UPPER)
        .step_by(REND_STEP as usize)
        .map(|i| {
            ShoreLandSearchForm::new(Some(i), Some(i + REND_STEP), Some(2), Some(2), 2021, 9, 6)
        });
    let records = stream::iter(forms)
        .map(|form| {
            let client = &client;
            async move { query(client, form).await }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    dbg!(records);
}

async fn query(
    client: &reqwest::Client,
    form: ShoreLandSearchForm,
) -> Result<Vec<(Vec<String>, Option<(u32, u32)>)>, Box<dyn std::error::Error>> {
    let pattern = Regex::new(r"^2BR - ([NS]\d{4})$").unwrap();
    let resp = client
        .post("https://macapartments.secure.force.com/mac/ShorelandSiteSearch")
        .form(&form)
        .send()
        .await?;
    let html = resp.text().await?;
    let parser = kuchiki::parse_html().one(html);

    let apartments = parser
        .select(r"form#shorelandsearch\:searchresultspanelform div.content-wrapper.clearfix")
        .unwrap();

    let records = future::join_all(apartments.map(|apartment| {
        let room = pattern
            .captures(
                apartment
                    .as_node()
                    .select_first("a.caps")
                    .unwrap()
                    .as_node()
                    .first_child()
                    .unwrap()
                    .as_text()
                    .unwrap()
                    .borrow()
                    .trim(),
            )
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();

        let url = format!(
            "https://www.macapartments.com/unit/Regents-Park-{}-2BR",
            room
        );

        let mut info = extract_info(&apartment);
        info.push(room);

        async move {
            match client.get(url).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(body) => (info, extract_price(body)),
                    Err(_) => (info, None),
                },
                Err(_) => (info, None),
            }
        }
    }))
    .await;

    Ok(records)
}

fn extract_info<T>(apartment: &kuchiki::NodeDataRef<T>) -> Vec<String> {
    apartment
        .as_node()
        .select("table.results-table > tbody > tr > td")
        .unwrap()
        .map(|info| {
            info.as_node()
                .first_child()
                .unwrap()
                .as_text()
                .unwrap()
                .borrow()
                .trim()
                .to_string()
        })
        .collect::<Vec<_>>()
}

fn extract_price(body: String) -> Option<(u32, u32)> {
    let price_pattern = Regex::new(r"^\$(\d+)\.00$").unwrap();
    let period_pattern = Regex::new(r"^for (\d+) Months$").unwrap();
    let parser = kuchiki::parse_html().one(body);
    let price_unit = parser.select_first(r"h3#priceUnit").unwrap();
    let price = price_pattern
        .captures(
            price_unit
                .as_node()
                .first_child()
                .unwrap()
                .as_text()
                .unwrap()
                .borrow()
                .trim(),
        )
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap();
    let period = period_pattern
        .captures(
            price_unit
                .as_node()
                .select_first("span")
                .unwrap()
                .as_node()
                .first_child()
                .unwrap()
                .as_text()
                .unwrap()
                .borrow()
                .trim(),
        )
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap();

    Some((price, period))
}
