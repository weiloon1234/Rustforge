use async_trait::async_trait;
use core_db::{
    common::sql::DbConn,
    generated::models::{CountryCol, CountryIsDefault, CountryModel, CountryStatus},
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

        for seed in countries {
            let seed = normalize_country_seed(seed);
            let status =
                CountryStatus::from_storage(default_country_status_for_iso2(&seed.iso2))
                    .ok_or_else(|| anyhow::anyhow!("invalid country status for {}", seed.iso2))?;
            let currencies = serde_json::to_value(seed.currencies)?;
            let is_default = if seed.iso2.eq_ignore_ascii_case("MY") {
                CountryIsDefault::Yes
            } else {
                CountryIsDefault::No
            };

            CountryModel::create(DbConn::pool(db))
                .set(CountryCol::ISO2, seed.iso2)?
                .set(CountryCol::ISO3, seed.iso3)?
                .set(CountryCol::ISO_NUMERIC, seed.iso_numeric)?
                .set(CountryCol::NAME, seed.name)?
                .set(CountryCol::OFFICIAL_NAME, seed.official_name)?
                .set(CountryCol::CAPITAL, seed.capital)?
                .set(CountryCol::CAPITALS, seed.capitals)?
                .set(CountryCol::REGION, seed.region)?
                .set(CountryCol::SUBREGION, seed.subregion)?
                .set(CountryCol::CURRENCIES, currencies)?
                .set(CountryCol::PRIMARY_CURRENCY_CODE, seed.primary_currency_code)?
                .set(CountryCol::CALLING_CODE, seed.calling_code)?
                .set(CountryCol::CALLING_ROOT, seed.calling_root)?
                .set(CountryCol::CALLING_SUFFIXES, seed.calling_suffixes)?
                .set(CountryCol::TLDS, seed.tlds)?
                .set(CountryCol::TIMEZONES, seed.timezones)?
                .set(CountryCol::LATITUDE, seed.latitude)?
                .set(CountryCol::LONGITUDE, seed.longitude)?
                .set(CountryCol::INDEPENDENT, seed.independent)?
                .set(CountryCol::STATUS, status)?
                .set(CountryCol::IS_DEFAULT, is_default)?
                .set(CountryCol::ASSIGNMENT_STATUS, seed.assignment_status)?
                .set(CountryCol::UN_MEMBER, seed.un_member)?
                .set(CountryCol::FLAG_EMOJI, seed.flag_emoji)?
                .on_conflict_update(&[CountryCol::ISO2])
                .save()
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "CountriesSeeder"
    }
}
