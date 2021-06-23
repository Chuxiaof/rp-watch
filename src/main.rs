use futures::{stream, Stream, StreamExt};
use kuchiki;
use kuchiki::traits::TendrilSink;
use lazy_static::lazy_static;
use regex::Regex;
use tokio;

use rp_watch::shorelandsearch::{BasicInfo, FullInfo, ShoreLandSearchForm};

lazy_static! {
    static ref BEDROOMS_PATTERN: Regex = Regex::new(r"^(\d+)$").unwrap();
    static ref BATHROOMS_PATTERN: Regex = Regex::new(r"^(\d+)\.0$").unwrap();
    static ref AREA_PATTERN: Regex = Regex::new(r"^(\d+)\.00$").unwrap();
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
    let client = reqwest::Client::new();
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
    let records = stream::iter(forms)
        .map(|form| get_page(&client, form))
        .buffer_unordered(CONCURRENT_REQUESTS)
        .flat_map(|page| page)
        .map(|info| complete(&client, info))
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    dbg!(&records);
    dbg!(records.len());
}

async fn get_page(
    client: &reqwest::Client,
    form: ShoreLandSearchForm,
) -> impl Stream<Item = BasicInfo> {
    println!("Fetching page for {:?}", form);
    let html = client
        .post("https://macapartments.secure.force.com/mac/ShorelandSiteSearch")
        .form(&form)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("Got page for {:?}", form);

    stream::iter(
        kuchiki::parse_html()
            .one(html)
            .select(r"form#shorelandsearch\:searchresultspanelform div.content-wrapper.clearfix")
            .unwrap()
            .map(|row| extract_info(&row)),
    )
}

fn extract_info<T>(row: &kuchiki::NodeDataRef<T>) -> BasicInfo {
    let mut info = BasicInfo::default();
    row.as_node()
        .select("table.results-table > tbody > tr > td")
        .unwrap()
        .map(|node| {
            node.as_node()
                .first_child()
                .unwrap()
                .as_text()
                .unwrap()
                .borrow()
                .trim()
                .to_string()
        })
        .enumerate()
        .for_each(|(i, x)| match i {
            0 => {
                info.bedrooms = BEDROOMS_PATTERN
                    .captures(&x)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
            }
            1 => {
                info.bathrooms = BATHROOMS_PATTERN
                    .captures(&x)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
            }
            2 => {
                info.area = AREA_PATTERN
                    .captures(&x)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
            }
            3 => {
                info.rent = RENT_PATTERN
                    .captures(&x)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
            }
            _ => panic!(),
        });

    info.url = format!(
        "https://www.macapartments.com/unit/Regents-Park-{}-2BR",
        ROOM_PATTERN
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
    );

    info
}

async fn complete(client: &reqwest::Client, info: BasicInfo) -> FullInfo {
    println!("Fetching {}", &info.url);
    let body = client
        .get(&info.url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("Got {}", &info.url);
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

    FullInfo {
        info: info,
        rent,
        period,
    }
}
