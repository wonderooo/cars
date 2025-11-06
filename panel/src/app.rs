use leptos::prelude::*;
use leptos::server_fn::serde::{Deserialize, Serialize};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/panel.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LotVehicleDto {
    pub lot_number: i32,
    pub make: String,
    pub year: i32,
}

#[server]
pub async fn query_db() -> Result<Vec<LotVehicleDto>, ServerFnError> {
    use common::persistence::models::copart::LotVehicle;
    use common::persistence::schema::lot_vehicle::dsl::lot_vehicle;
    use common::persistence::PG_POOL;
    use diesel::query_dsl::QueryDsl;
    use diesel::SelectableHelper;
    use diesel_async::RunQueryDsl;

    let mut conn = PG_POOL.get().await?;
    Ok(lot_vehicle
        .select(LotVehicle::as_select())
        .load(&mut conn)
        .await?
        .into_iter()
        .map(|v| LotVehicleDto {
            lot_number: v.lot_number,
            make: v.make,
            year: v.year,
        })
        .collect())
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let vehicles = OnceResource::new(query_db());

    view! {
        <div>
            <h1>"Static Data Example with Suspense"</h1>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view! {
                    <p class="text-red-500">"Error: " {format!("{:?}", errors)}</p>
                }>
                    <ul>
                        {move || {
                            match vehicles.get() {
                                Some(Ok(vehicles)) => {
                                    view! {
                                        <>
                                            {vehicles.into_iter()
                                                .map(|v| view! { <li>{v.year}</li> })
                                                .collect::<Vec<_>>()}
                                        </>
                                    }.into_any()
                                }
                                Some(Err(e)) => {
                                    view! { <li>"Failed to load vehicles."</li> }.into_any()
                                }
                                None => {
                                    view! { <li>"No data available."</li> }.into_any()
                                }
                            }
                        }}
                    </ul>
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
