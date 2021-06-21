use rp_watch::shorelandsearch::ShoreLandSearchForm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let form = ShoreLandSearchForm::new(Some(2500), None, Some(2), Some(2), 2021, 9, 6);
    let resp = reqwest::blocking::Client::new()
        .post("https://macapartments.secure.force.com/mac/ShorelandSiteSearch")
        .form(&form)
        .send()?;

    println!("{}", resp.text()?);

    Ok(())
}
