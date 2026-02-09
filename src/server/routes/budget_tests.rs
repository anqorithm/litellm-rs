use super::*;
use actix_web::{App, test};

#[actix_web::test]
async fn test_set_provider_budget() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let request = SetProviderBudgetRequest {
        provider: "openai".to_string(),
        max_budget: 1000.0,
        reset_period: ResetPeriod::Monthly,
        soft_limit_percentage: 0.8,
        currency: Currency::USD,
        enabled: true,
    };

    let req = test::TestRequest::post()
        .uri("/v1/budget/providers")
        .set_json(&request)
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_set_provider_budget_validation() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    // Test empty provider name
    let request = SetProviderBudgetRequest {
        provider: "".to_string(),
        max_budget: 1000.0,
        reset_period: ResetPeriod::Monthly,
        soft_limit_percentage: 0.8,
        currency: Currency::USD,
        enabled: true,
    };

    let req = test::TestRequest::post()
        .uri("/v1/budget/providers")
        .set_json(&request)
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Test negative budget
    let request = SetProviderBudgetRequest {
        provider: "openai".to_string(),
        max_budget: -100.0,
        reset_period: ResetPeriod::Monthly,
        soft_limit_percentage: 0.8,
        currency: Currency::USD,
        enabled: true,
    };

    let req = test::TestRequest::post()
        .uri("/v1/budget/providers")
        .set_json(&request)
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_list_provider_budgets() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    budget_limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
    );
    budget_limits.providers.set_provider_limit(
        "anthropic",
        ProviderLimitConfig::new(500.0, ResetPeriod::Monthly),
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/v1/budget/providers")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_provider_budget() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    budget_limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/v1/budget/providers/openai")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_provider_budget_not_found() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/v1/budget/providers/nonexistent")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_delete_provider_budget() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    budget_limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/v1/budget/providers/openai")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_set_model_budget() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let request = SetModelBudgetRequest {
        model: "gpt-4".to_string(),
        max_budget: 500.0,
        reset_period: ResetPeriod::Monthly,
        soft_limit_percentage: 0.8,
        currency: Currency::USD,
        enabled: true,
    };

    let req = test::TestRequest::post()
        .uri("/v1/budget/models")
        .set_json(&request)
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_list_model_budgets() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    budget_limits
        .models
        .set_model_limit("gpt-4", ModelLimitConfig::new(500.0, ResetPeriod::Monthly));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/v1/budget/models")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_budget_summary() {
    let budget_limits = Arc::new(UnifiedBudgetLimits::new());
    budget_limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
    );
    budget_limits
        .models
        .set_model_limit("gpt-4", ModelLimitConfig::new(500.0, ResetPeriod::Monthly));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(budget_limits))
            .configure(configure_budget_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/v1/budget/summary")
        .to_request();

    let resp: actix_web::dev::ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
