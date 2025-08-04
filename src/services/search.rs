use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{query_as_with, PgPool, FromRow, postgres::PgArguments};
use sqlx::Arguments;
use bigdecimal::ToPrimitive;
 use sqlx::types::BigDecimal;

#[derive(Deserialize)]
pub struct ProductQuery {
    search: Option<String>, // unused for now
    brands: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    random: Option<bool>,
    viewed: Option<String>,
}

#[derive(Serialize)]
pub struct LaptopResponse {
    id: String,
    image: Option<String>,
    display_name: String,
    show_price: String,
    tag: String,
}

#[derive( FromRow, Clone)]
pub struct Laptop {
    id: i32,
    display_name: String,
    brand_name: String,
    model_name: String,
    model_year: Option<i32>,
    product_authentication: Option<String>,
    product_type: Option<String>,
    processor: Option<String>,
    processor_generation: Option<String>,
    processor_series: Option<String>,
    ram: Option<i32>,
    ram_tpe: Option<String>,
    storage: Option<i32>,
    storage_type: Option<String>,
    graphic: Option<String>,
    graphic_ram: Option<i32>,
    battery: Option<String>,
    touchscreen: Option<bool>,
    show_price: BigDecimal,
    face_image_url: Option<String>,
}

#[allow(unused_assignments)]
#[get("/api/productshow/getproduct")]
async fn get_filtered_products(
    pool: web::Data<PgPool>,
    query: web::Query<ProductQuery>,
) -> impl Responder {
    if query.random.unwrap_or(false) {
    return match get_random_laptops(pool.get_ref(), &query).await {
        Ok(results) => {
            let response_vec: Vec<LaptopResponse> = results.into_iter()
                .map(map_to_response)
                .collect();
            HttpResponse::Ok().json(response_vec)
        },
        Err(err) => {
                eprintln!("Random fetch error: {:?}", err);
                HttpResponse::InternalServerError().body("Failed to fetch random laptops")
            }
    };
}
    if let Some(viewed_ids) = &query.viewed {
    return match recommendation_list(pool.get_ref(), viewed_ids, &query).await {
        Ok(results) => {
            let response_vec: Vec<LaptopResponse> = results.into_iter()
                .map(map_to_response)
                .collect();
            HttpResponse::Ok().json(response_vec)
        },
        Err(err) => {
                eprintln!("Recommendation error: {:?}", err);
                HttpResponse::InternalServerError().body("Failed to fetch recommendations")
            }
    };
}

    let mut sql = String::from("SELECT id, brand_name, model_name, show_price, face_image_url FROM laptop_details WHERE 1=1");
    let mut args = PgArguments::default();
    let mut param_index = 1;

    if let Some(brands) = &query.brands {
        let brand_list: Vec<&str> = brands.split(',').collect();
        if !brand_list.is_empty() {
            sql.push_str(" AND (");
            for (i, brand) in brand_list.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" OR ");
                }
                sql.push_str(&format!("brand_name ILIKE ${}", param_index));
                args.add(format!("%{}%", brand));
                param_index += 1;
            }
            sql.push(')');
        }
    }

    if let Some(min_price) = query.min_price {
        sql.push_str(&format!(" AND show_price >= ${}", param_index));
        args.add(min_price);
        param_index += 1;
    }

    if let Some(max_price) = query.max_price {
        sql.push_str(&format!(" AND show_price <= ${}", param_index));
        args.add(max_price);
        param_index += 1;
    }

    let laptops = query_as_with::<_, Laptop, _>(&sql, args)
        .fetch_all(pool.get_ref())
        .await;
match laptops {
    Ok(results) if !results.is_empty() => {
        let response_vec: Vec<LaptopResponse> = results.into_iter()
            .map(map_to_response)
            .collect();
        HttpResponse::Ok().json(response_vec)
    },
    _ => HttpResponse::Ok().json(Vec::<LaptopResponse>::new()),
}
}

#[allow(unused_assignments)]
async fn get_random_laptops(
    pool: &PgPool,
    query: &ProductQuery,
) -> Result<Vec<Laptop>, sqlx::Error> {
    let mut sql = String::from(
    "SELECT id, display_name, brand_name, model_name, model_year, product_authentication, product_type,
       processor, processor_generation, processor_series, ram, ram_type,
       storage, storage_type, graphic, graphic_ram, battery, touchscreen, show_price, face_image_url
 FROM laptop_details WHERE 1=1",
);
    let mut args = PgArguments::default();
    let mut param_index = 1;

    if let Some(brands) = &query.brands {
        let brand_list: Vec<&str> = brands.split(',').collect();
        if !brand_list.is_empty() {
            sql.push_str(" AND (");
            for (i, brand) in brand_list.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" OR ");
                }
                sql.push_str(&format!("brand_name ILIKE ${}", param_index));
                args.add(format!("%{}%", brand));
                param_index += 1;
            }
            sql.push(')');
        }
    }

    if let Some(min_price) = query.min_price {
        sql.push_str(&format!(" AND show_price >= ${}", param_index));
        args.add(min_price);
        param_index += 1;
    }

    if let Some(max_price) = query.max_price {
        sql.push_str(&format!(" AND show_price <= ${}", param_index));
        args.add(max_price);
        param_index += 1;
    }

    sql.push_str(" ORDER BY RANDOM() LIMIT 16");

    let laptops = query_as_with::<_, Laptop, _>(&sql, args)
        .fetch_all(pool)
        .await?;

    Ok(laptops)
}

#[allow(unused_assignments)]
async fn recommendation_list(
    pool: &PgPool,
    viewed_ids: &str,
    query: &ProductQuery,
) -> Result<Vec<Laptop>, sqlx::Error> {
    let ids: Vec<i32> = viewed_ids
        .split(',')
        .filter_map(|id| id.trim().parse::<i32>().ok())
        .collect();

    if ids.is_empty() {
        return get_random_laptops(pool, query).await;
    }

    let placeholders = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(",");

  let sql = format!(
    "SELECT id, display_name, brand_name, model_name, model_year, product_authentication, product_type,
       processor, processor_generation, processor_series, ram, ram_type,
       storage, storage_type, graphic, graphic_ram, battery, touchscreen, show_price, face_image_url
 FROM laptop_details WHERE id IN ({})",
    placeholders
);

    let mut args = PgArguments::default();
    for id in &ids {
        args.add(*id);
    }

    let viewed_laptops = query_as_with::<_, Laptop, _>(&sql, args)
        .fetch_all(pool)
        .await?;

    if viewed_laptops.is_empty() {
        return get_random_laptops(pool, query).await;
    }

    let mut filter_sql =
        String::from("SELECT id, display_name, brand_name, model_name, model_year, product_authentication, product_type,
       processor, processor_generation, processor_series, ram, ram_type,
       storage, storage_type, graphic, graphic_ram, battery, touchscreen, show_price, face_image_url
    FROM laptop_details WHERE 1=1");
    let mut filter_args = PgArguments::default();
    let mut param_index = 1;

    if let Some(brands) = &query.brands {
        let brand_list: Vec<&str> = brands.split(',').collect();
        if !brand_list.is_empty() {
            filter_sql.push_str(" AND (");
            for (i, brand) in brand_list.iter().enumerate() {
                if i > 0 {
                    filter_sql.push_str(" OR ");
                }
                filter_sql.push_str(&format!("brand_name ILIKE ${}", param_index));
                filter_args.add(format!("%{}%", brand));
                param_index += 1;
            }
            filter_sql.push(')');
        }
    }

    if let Some(min_price) = query.min_price {
        filter_sql.push_str(&format!(" AND show_price >= ${}", param_index));
        filter_args.add(min_price);
        param_index += 1;
    }

    if let Some(max_price) = query.max_price {
        filter_sql.push_str(&format!(" AND show_price <= ${}", param_index));
        filter_args.add(max_price);
        param_index += 1;
    }

    let all_candidates = query_as_with::<_, Laptop, _>(&filter_sql, filter_args)
        .fetch_all(pool)
        .await?;

    let mut scored: Vec<(Laptop, f64)> = all_candidates
    .into_iter()
    .map(|laptop| {
        let mut total_score = 0.0;

        for viewed in &viewed_laptops {
            let mut score = 0.0;

            // Exact match categorical fields
            if laptop.brand_name == viewed.brand_name {
                score += 1.0;
            }
            if laptop.model_name == viewed.model_name {
                score += 1.0;
            }
            if laptop.product_type == viewed.product_type {
                score += 0.8;
            }
            if laptop.processor == viewed.processor {
                score += 1.0;
            }
            if laptop.processor_generation == viewed.processor_generation {
                score += 0.8;
            }
            if laptop.processor_series == viewed.processor_series {
                score += 0.8;
            }
            if laptop.ram_tpe == viewed.ram_tpe {
                score += 0.6;
            }
            if laptop.storage_type == viewed.storage_type {
                score += 0.6;
            }
            if laptop.graphic == viewed.graphic {
                score += 0.6;
            }
            if laptop.battery == viewed.battery {
                score += 0.6;
            }
            if laptop.touchscreen == viewed.touchscreen {
                score += 0.6;
            }

            // Numeric similarity
            let price_diff = (&laptop.show_price - &viewed.show_price).abs();
            let price_diff_f64 = price_diff.to_f64().unwrap_or(0.0);
            score += 1.0 / (1.0 + price_diff_f64 / 10000.0);

            if let (Some(r1), Some(r2)) = (laptop.ram, viewed.ram) {
                score += 1.0 / (1.0 + ((r1 - r2).abs() as f64) / 4.0);
            }

            if let (Some(s1), Some(s2)) = (laptop.storage, viewed.storage) {
                score += 1.0 / (1.0 + ((s1 - s2).abs() as f64) / 256.0);
            }

            if let (Some(g1), Some(g2)) = (laptop.graphic_ram, viewed.graphic_ram) {
                score += 1.0 / (1.0 + ((g1 - g2).abs() as f64) / 2.0);
            }

            if let (Some(y1), Some(y2)) = (laptop.model_year, viewed.model_year) {
                score += 1.0 / (1.0 + ((y1 - y2).abs() as f64));
            }

            total_score += score;
        }

        total_score /= viewed_laptops.len() as f64;
        (laptop, total_score)
    })
    .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let top_results: Vec<Laptop> = scored
        .into_iter()
        .take(16)
        .map(|(laptop, _)| laptop)
        .collect();

    Ok(top_results)
}

fn map_to_response(laptop: Laptop) -> LaptopResponse {
    let display_name = laptop.display_name;
    let tag = laptop
        .product_authentication
        .clone()
        .unwrap_or_else(|| "Performance Laptop".to_string());

    LaptopResponse {
        id: laptop.id.to_string(),
        image: laptop.face_image_url,
        display_name,
        show_price: format!("{:.2}", laptop.show_price.to_f64().unwrap_or(0.0)),
        tag,
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_filtered_products);
}