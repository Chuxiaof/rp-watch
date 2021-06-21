use serde::ser::{Serialize, SerializeStruct, Serializer};

pub struct ShoreLandSearchForm {
    min_rent: Option<u32>,
    max_rent: Option<u32>,
    bedrooms: Option<u8>,
    bathrooms: Option<u8>,
    move_in_date: String,
}

impl ShoreLandSearchForm {
    pub fn new(
        min_rent: Option<u32>,
        max_rent: Option<u32>,
        bedrooms: Option<u8>,
        bathrooms: Option<u8>,
        move_in_year: u32,
        move_in_month: u32,
        move_in_day: u32,
    ) -> Self {
        Self {
            min_rent,
            max_rent,
            bedrooms,
            bathrooms,
            move_in_date: format!("{}/{}/{}", move_in_month, move_in_day, move_in_year),
        }
    }
}

impl Serialize for ShoreLandSearchForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ShoreLandSearchForm", 10)?;
        state.serialize_field("shorelandsearch:mainform", "shorelandsearch:mainform")?;

        match self.min_rent {
            Some(min_rent) => {
                state.serialize_field("shorelandsearch:mainform:min_rent", &min_rent)?
            }
            None => state.serialize_field("shorelandsearch:mainform:min_rent", "")?,
        }
        match self.max_rent {
            Some(max_rent) => {
                state.serialize_field("shorelandsearch:mainform:max_rent", &max_rent)?
            }
            None => state.serialize_field("shorelandsearch:mainform:max_rent", "")?,
        }
        match self.bedrooms {
            Some(bedrooms) => state.serialize_field(
                "shorelandsearch:mainform:bedrooms",
                &format!("{} Bedrooms", bedrooms),
            )?,
            None => state.serialize_field("shorelandsearch:mainform:bedrooms", "")?,
        }
        match self.bathrooms {
            Some(bathrooms) => {
                state.serialize_field("shorelandsearch:mainform:bathrooms", &bathrooms)?
            }
            None => state.serialize_field("shorelandsearch:mainform:bathrooms", "")?,
        }
        state.serialize_field("move_in_date", &self.move_in_date)?;

        state.serialize_field("shorelandsearch:mainform:search", "Search")?;
        state.serialize_field("com.salesforce.visualforce.ViewState", "i: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")?;
        state.serialize_field(
            "com.salesforce.visualforce.ViewStateVersion",
            "202106172003060553",
        )?;
        state.serialize_field("com.salesforce.visualforce.ViewStateMAC", "AGV5SnViMjVqWlNJNkluSkJUMFpWY1ZKTlQzTjRTVGxuWWxKeVNsRnZaREpQU21GcVNHeE5RMVJrYUd0NWRtdDBlVWR2VmxGY2RUQXdNMlFpTENKMGVYQWlPaUpLVjFRaUxDSmhiR2NpT2lKSVV6STFOaUlzSW10cFpDSTZJbnRjSW5SY0lqcGNJakF3Ukdrd01EQXdNREF3UzJOVGJWd2lMRndpZGx3aU9sd2lNREpITlVFd01EQXdNREJaT0Uwd1hDSXNYQ0poWENJNlhDSjJabk5wWjI1cGJtZHJaWGxjSWl4Y0luVmNJanBjSWpBd05Xa3dNREF3TURBeVdFVmFjMXdpZlNJc0ltTnlhWFFpT2xzaWFXRjBJbDBzSW1saGRDSTZNVFl5TkRJNU1USTFPVGt3TkN3aVpYaHdJam93ZlE9PS4uaVRpaDNJR3gyd3AtSGxPcS04SHdoZzRuVUh3cTNiLWNwaDBoWXI0RG1fWT0=")?;

        state.end()
    }
}
