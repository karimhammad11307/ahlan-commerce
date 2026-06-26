use serde::Deserialize;
use crate::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct ExternalProductPayload {
    pub name: String,
    pub slug: String,
    pub body_html: Option<String>,
    pub price: String,
    #[serde(alias = "stock")]
    pub qty: u32,
    #[serde(alias = "is_visible")]
    pub is_active: Option<bool>,
}

pub fn adapt_external_product(payload: ExternalProductPayload) -> Result<catalog::ProductCreate, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::ValidationFailed("name cannot be empty".to_string()));
    }
    
    if payload.slug.trim().is_empty() {
        return Err(AppError::ValidationFailed("slug cannot be empty".to_string()));
    }
    
    let price_f64 = payload.price.parse::<f64>().unwrap_or(-1.0);
    if price_f64 < 0.0 {
        return Err(AppError::ValidationFailed("price cannot be negative or invalid".to_string()));
    }

    let description = payload.body_html.and_then(|s| {
        if s.trim().is_empty() {
            None
        } else {
            Some(s)
        }
    });

    let price_cents = (price_f64 * 100.0).round() as u32;
    let published = payload.is_active.unwrap_or(false);

    Ok(catalog::ProductCreate {
        title: payload.name,
        handle: payload.slug,
        description,
        price_cents,
        inventory_quantity: payload.qty,
        published,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conversion() {
        let payload = ExternalProductPayload {
            name: "Coffee Mug".to_string(),
            slug: "coffee-mug".to_string(),
            body_html: Some("Ceramic mug".to_string()),
            price: "25.999".to_string(),
            qty: 12,
            is_active: Some(true),
        };

        let result = adapt_external_product(payload).unwrap();
        assert_eq!(result.title, "Coffee Mug");
        assert_eq!(result.handle, "coffee-mug");
        assert_eq!(result.description, Some("Ceramic mug".to_string()));
        assert_eq!(result.price_cents, 2600); // rounded from 2599.9
        assert_eq!(result.inventory_quantity, 12);
        assert_eq!(result.published, true);
    }

    #[test]
    fn test_empty_body_html_normalizes_to_none() {
        let payload = ExternalProductPayload {
            name: "Mug".to_string(),
            slug: "mug".to_string(),
            body_html: Some("   ".to_string()),
            price: "10.0".to_string(),
            qty: 5,
            is_active: None,
        };

        let result = adapt_external_product(payload).unwrap();
        assert_eq!(result.description, None);
        assert_eq!(result.published, false); // default
    }

    #[test]
    fn test_blank_name_fails() {
        let payload = ExternalProductPayload {
            name: "   ".to_string(),
            slug: "mug".to_string(),
            body_html: None,
            price: "10.0".to_string(),
            qty: 5,
            is_active: None,
        };

        let err = match adapt_external_product(payload) {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => e,
        };
        assert!(matches!(err, AppError::ValidationFailed(_)));
    }

    #[test]
    fn test_negative_price_fails() {
        let payload = ExternalProductPayload {
            name: "Mug".to_string(),
            slug: "mug".to_string(),
            body_html: None,
            price: "-5.0".to_string(),
            qty: 5,
            is_active: None,
        };

        let err = match adapt_external_product(payload) {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => e,
        };
        assert!(matches!(err, AppError::ValidationFailed(_)));
    }
}
