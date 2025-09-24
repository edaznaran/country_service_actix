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
                    Err(e) => serde_json::json!({"message": e.to_string()}),
                }
            }
            "createCountry" => {
                let create_request = serde_json::from_value::<CreateCountryRequest>(payload)
                    .expect("Invalid create country payload");
                match self.create_country(create_request) {
                    Ok(created_country) => serde_json::json!(created_country),
                    Err(e) => serde_json::json!({"message": e.to_string()}),
                }
            }
            "updateCountry" => {
                let country_id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<i32>().ok())
                    .expect("Missing or invalid country ID");
                let update_payload = payload
                    .get("updateCountryDto")
                    .and_then(|v| serde_json::from_value::<UpdateCountryRequest>(v.clone()).ok())
                    .expect("Invalid update country payload");
                match self.update_country(country_id, update_payload) {
                    Ok(updated_country) => serde_json::json!(updated_country),
                    Err(e) => serde_json::json!({"message": e.to_string()}),
                }
            }
            "removeCountry" => {
                let country_id = payload
                    .as_str()
                    .and_then(|v| v.parse::<i32>().ok())
                    .expect("Missing or invalid country ID");
                match self.delete_country(country_id) {
                    Ok(deleted_count) => serde_json::json!({"deleted": deleted_count}),
                    Err(e) => serde_json::json!({"message": e.to_string()}),
                }
            }
            _ => {
                println!("Unknown command");
                serde_json::json!({"message": "Unknown command"})
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
        let mut duplicate_messages = Vec::new();
        let existing_by_code: Option<Country> = country
            .filter(code.eq(&new_country.code))
            .first(self.conn)
            .optional()?;
        if existing_by_code.is_some() {
            duplicate_messages.push(format!(
                "Country with code {} already exists",
                new_country.code
            ));
        }
        let existing_by_name: Option<Country> = country
            .filter(name.eq(&new_country.name))
            .first(self.conn)
            .optional()?;
        if existing_by_name.is_some() {
            duplicate_messages.push(format!(
                "Country with name {} already exists",
                new_country.name
            ));
        }
        let existing_by_dial_code: Option<Country> = country
            .filter(dial_code.eq(&new_country.dial_code))
            .first(self.conn)
            .optional()?;
        if existing_by_dial_code.is_some() {
            duplicate_messages.push(format!(
                "Country with dial code {} already exists",
                new_country.dial_code
            ));
        }
        if !duplicate_messages.is_empty() {
            return Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new(duplicate_messages.join(", ")),
            ));
        }
        diesel::insert_into(country)
            .values((
                name.eq(new_country.name),
                code.eq(new_country.code),
                dial_code.eq(new_country.dial_code),
            ))
            .returning(Country::as_returning())
            .get_result(self.conn)
    }

    fn update_country(
        &mut self,
        _id: i32,
        _payload: UpdateCountryRequest,
    ) -> Result<Country, diesel::result::Error> {
        // Lógica para actualizar un país
        let target = country.filter(id.eq(_id));
        let updated_country = _payload;

        diesel::update(target)
            .set((
                updated_country.name.map(|n| name.eq(n)),
                updated_country.code.map(|c| code.eq(c)),
                updated_country.dial_code.map(|d| dial_code.eq(d)),
            ))
            .get_result(self.conn)
    }

    fn delete_country(&mut self, _id: i32) -> Result<usize, diesel::result::Error> {
        // Lógica para eliminar un país
        let target = country.filter(id.eq(_id));
        diesel::delete(target).execute(self.conn)
    }
}
