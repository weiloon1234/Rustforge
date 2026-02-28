pub mod admin_bootstrap_seeder;
pub mod countries_seeder;

pub fn register_seeders(seeders: &mut Vec<Box<dyn core_db::seeder::Seeder>>) {
    seeders.push(Box::new(countries_seeder::CountriesSeeder));
    seeders.push(Box::new(admin_bootstrap_seeder::AdminBootstrapSeeder));
}
