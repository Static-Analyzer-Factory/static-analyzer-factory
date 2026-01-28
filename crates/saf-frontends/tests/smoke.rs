use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;

#[test]
fn air_json_frontend_has_correct_id() {
    let frontend = AirJsonFrontend;
    assert_eq!(frontend.frontend_id(), "air-json");
}
