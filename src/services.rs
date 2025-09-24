use crate::models::Country;
use crate::schema::country::dsl::*;
use diesel::PgConnection;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::Value;

pub struct CountryController<'a> {
    pub conn: &'a mut PgConnection,
}

#[derive(Deserialize, Default)]
struct CountryQuery {
    name: Option<String>,
    code: Option<String>,
    dial_code: Option<String>,
}

// ESTRUCTURA PARA EL PAYLOAD DE CREACIÓN DE PAÍS (campos obligatorios)
#[derive(serde::Deserialize, serde::Serialize)]
struct CreateCountryRequest {
    name: String,
    code: String,
    dial_code: String,
}

// ESTRUCTURA PARA EL PAYLOAD DE ACTUALIZACIÓN DE PAÍS (campos opcionales)
#[derive(serde::Deserialize, serde::Serialize)]
struct UpdateCountryRequest {
    name: Option<String>,
    code: Option<String>,
    dial_code: Option<String>,
}

impl<'a> CountryController<'a> {
    pub fn controller(&mut self, cmd: &str, payload: Value) -> Value {
        match cmd {
            "findByCriteria" => {
                let country_query = serde_json::from_value::<CountryQuery>(payload)
                    .unwrap_or_else(|_| CountryQuery::default());
                match self.get_countries(country_query) {
                    Ok(countries) => serde_json::json!(countries),
                    Err(e) => serde_json::json!({"error": e.to_string()}),
                }
            }
            "createCountry" => {
                let create_request = serde_json::from_value::<CreateCountryRequest>(payload)
                    .expect("Invalid create country payload");
                match self.create_country(create_request) {
                    Ok(created_country) => serde_json::json!(created_country),
                    Err(e) => serde_json::json!({"error": e.to_string()}),
                }
            }
            _ => {
                println!("Unknown command");
                serde_json::json!({"error": "Unknown command"})
            }
        }
    }

    fn get_countries(
        &mut self,
        _payload: CountryQuery,
    ) -> Result<Vec<Country>, diesel::result::Error> {
        let mut query = country.into_boxed();

        if let Some(ref n) = _payload.name {
            query = query.filter(name.ilike(format!("%{}%", n)));
        }
        if let Some(ref c) = _payload.code {
            query = query.filter(code.like(format!("%{}%", c)));
        }
        if let Some(ref d) = _payload.dial_code {
            query = query.filter(dial_code.ilike(format!("%{}%", d)));
        }

        query.load(self.conn)
    }

    fn create_country(
        &mut self,
        _payload: CreateCountryRequest,
    ) -> Result<Country, diesel::result::Error> {
        // Lógica para crear un país
        let new_country = _payload;
        diesel::insert_into(country)
            .values((
                name.eq(new_country.name),
                code.eq(new_country.code),
                dial_code.eq(new_country.dial_code),
            ))
            .returning(Country::as_returning())
            .get_result(self.conn)
    }
}
