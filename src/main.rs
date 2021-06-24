use futures::{stream, Stream, StreamExt};
use kuchiki;
use kuchiki::traits::TendrilSink;
use lazy_static::lazy_static;
use regex::Regex;
use tokio;

use rp_watch::shorelandsearch::{BareInfo, FullInfo, ShoreLandSearchForm};

lazy_static! {
    static ref INFO_PATTERN: Vec<Regex> = vec![
        Regex::new(r"^(\d+)$").unwrap(),
        Regex::new(r"^(\d+)\.0$").unwrap(),
        Regex::new(r"^(\d+)\.00$").unwrap(),
        Regex::new(r"^\$(\d+)\.00$").unwrap(),
    ];
    static ref RENT_PATTERN: Regex = Regex::new(r"^\$(\d+)\.00$").unwrap();
    static ref ROOM_PATTERN: Regex = Regex::new(r"^2BR - ([NS]\d{4})$").unwrap();
    static ref PERIOD_PATTERN: Regex = Regex::new(r"^for (\d+) Months$").unwrap();
}

static RENT_LOWER: u32 = 1700;
static RENT_UPPER: u32 = 2500;
static REND_STEP: u32 = 10;
static CONCURRENT_REQUESTS: usize = 16;

#[tokio::main]
async fn main() {
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let forms = (RENT_LOWER..RENT_UPPER)
        .step_by(REND_STEP as usize)
        .map(|i| {
            ShoreLandSearchForm::new(
                Some(i),
                Some(i + REND_STEP - 1),
                Some(2),
                Some(2),
                2021,
                9,
                6,
            )
        });
    let mut records = stream::iter(forms)
        .map(|form| get_page(&client, form))
        .buffer_unordered(CONCURRENT_REQUESTS)
        .flat_map(|page| page)
        .map(|info| complete(&client, info))
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;
    records.sort_by_key(|record| record.bare.room.clone());

    println!("room,bedrooms,bathrooms,area,wrong_rent,rent,period");
    for record in records {
        let bare = record.bare;
        println!(
            "{},{},{},{},{},{},{}",
            bare.room,
            bare.bedrooms,
            bare.bathrooms,
            bare.area,
            bare.rent,
            record.rent,
            record.period
        );
    }
}

async fn get_page(
    client: &reqwest::Client,
    form: ShoreLandSearchForm,
) -> impl Stream<Item = BareInfo> {
    let html = client
        .post("https://macapartments.secure.force.com/mac/ShorelandSiteSearch")
        .form(&form.data)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    stream::iter(
        kuchiki::parse_html()
            .one(html)
            .select(r"form#shorelandsearch\:searchresultspanelform div.content-wrapper.clearfix")
            .unwrap()
            .map(|row| extract_info(&row)),
    )
}

fn extract_info<T>(row: &kuchiki::NodeDataRef<T>) -> BareInfo {
    let mut info_iter = row
        .as_node()
        .select("table.results-table > tbody > tr > td")
        .unwrap()
        .zip(INFO_PATTERN.iter())
        .map(|(node, pattern)| {
            pattern
                .captures(
                    node.as_node()
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
                .parse()
                .unwrap()
        });

    BareInfo {
        room: ROOM_PATTERN
            .captures(
                row.as_node()
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
            .to_string(),
        bedrooms: info_iter.next().unwrap(),
        bathrooms: info_iter.next().unwrap(),
        area: info_iter.next().unwrap(),
        rent: info_iter.next().unwrap(),
    }
}

async fn complete(client: &reqwest::Client, bare: BareInfo) -> FullInfo {
    eprintln!("Fetching {}", &bare.room);
    let body = client
        .get(format!(
            "https://www.macapartments.com/unit/Regents-Park-{}-2BR",
            &bare.room
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let parser = kuchiki::parse_html().one(body);
    let rent_unit = parser.select_first(r"h3#priceUnit").unwrap();
    let rent = RENT_PATTERN
        .captures(
            rent_unit
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
    let period = PERIOD_PATTERN
        .captures(
            rent_unit
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

    FullInfo { bare, rent, period }
}
