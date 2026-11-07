// Our class for handling NSServices messages.
@interface CognixServicesProvider : NSObject
@end

// Functions implemented in Rust.
id cognix_services_provider_custom_url_scheme();
void cognix_app_open_urls(id app, id urls);
