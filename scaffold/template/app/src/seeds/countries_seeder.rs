use async_trait::async_trait;
use core_db::{
    common::sql::DbConn,
    generated::models::{Country, CountryCol, CountryStatus},
    platform::countries::{
        default_country_status_for_iso2, load_builtin_country_seed, normalize_country_seed,
    },
    seeder::Seeder,
};

#[derive(Debug, Default)]
pub struct CountriesSeeder;

#[async_trait]
impl Seeder for CountriesSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        let countries = load_builtin_country_seed()?;
        let model = Country::new(DbConn::pool(db), None);

        for seed in countries {
            let seed = normalize_country_seed(seed);
            let status =
                CountryStatus::from_storage(default_country_status_for_iso2(&seed.iso2))
                    .ok_or_else(|| anyhow::anyhow!("invalid country status for {}", seed.iso2))?;
            let currencies = serde_json::to_value(seed.currencies)?;

            model
                .insert()
                .set_iso2(seed.iso2)
                .set_iso3(seed.iso3)
                .set_iso_numeric(seed.iso_numeric)
                .set_name(seed.name)
                .set_official_name(seed.official_name)
                .set_capital(seed.capital)
                .set_capitals(seed.capitals)
                .set_region(seed.region)
                .set_subregion(seed.subregion)
                .set_currencies(currencies)
                .set_primary_currency_code(seed.primary_currency_code)
                .set_calling_code(seed.calling_code)
                .set_calling_root(seed.calling_root)
                .set_calling_suffixes(seed.calling_suffixes)
                .set_tlds(seed.tlds)
                .set_timezones(seed.timezones)
                .set_latitude(seed.latitude)
                .set_longitude(seed.longitude)
                .set_independent(seed.independent)
                .set_status(status)
                .set_assignment_status(seed.assignment_status)
                .set_un_member(seed.un_member)
                .set_flag_emoji(seed.flag_emoji)
                .on_conflict_update(&[CountryCol::Iso2])
                .save()
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "CountriesSeeder"
    }
}
