use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use std::panic;
use std::time::Duration;
use wasm_timer::Delay;
use url::Url;
extern crate console_error_panic_hook;
extern crate csv;
extern crate reqwest_wasm;
extern crate rand;

use rand::Rng;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct CityName {
    prefecture: String,
    city: String,
    prefecture_kana: String,
    city_kana: String,
    latitude: f64,
    longitude: f64,
}

static mut RECORDS: Vec<CityName> = vec![];

pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
    wasm_bindgen_futures::spawn_local(_main());
}

async fn _main() {
    unsafe { RECORDS = fetch().await.unwrap(); }
    match start().await {
      Err(_) => log::error!("Error"),
      Ok(_) => (),
    };
}

async fn start() -> Result<(), JsValue> {
    Delay::new(Duration::from_millis(3000)).await.unwrap();
    loop {
        let document = web_sys::window().unwrap().document().unwrap();
        let height = document.body().unwrap().client_height();
        append_text_after(1000, 10 * (height as u64) / 800).await;
    }
}

async fn append_text_after(millis: u64, n: u64) {
    Delay::new(Duration::from_millis(millis)).await.unwrap();
    for _i in 1..=n {
        append_text().await;
    }
}

async fn append_text() {
    let document = web_sys::window().unwrap().document().unwrap();
    let contents = document.get_element_by_id("citiescrowl_contents").unwrap();
    let element: web_sys::HtmlElement = contents.dyn_into::<web_sys::HtmlElement>().unwrap();
    let label = document.create_element("label").unwrap();
    element.append_child(&label).unwrap();

    let mut rng = rand::thread_rng();
    let n = rng.gen_range(1..=4);
    let s = rng.gen_range(0.8..=1.5);
    let y = rng.gen_range(-5.0..=100.0);

    label.set_class_name(format!("citiescrowl_text citiescrowl_text{}", n).as_str());
    label.set_attribute("for", "trigger").unwrap();
    label.set_attribute(
        "style",
        format!(
            "top: {}vh; animation-duration: {}s;",
            y,
            s * (1.0 + (n as f64) / 20.0) * 10.0).as_str()
        ).unwrap();

    unsafe {
        let i = rng.gen_range(0..RECORDS.len());
        label.set_inner_html(&RECORDS[i].city);

        let closure:Closure<dyn FnMut(_)> = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            let document = web_sys::window().unwrap().document().unwrap();
            let city_name = RECORDS[i].city.split("（").collect::<Vec<_>>()[0];
            let city_text = document.get_element_by_id("popup_content_city").unwrap();
            let prefecture_text = document.get_element_by_id("popup_content_prefecture").unwrap();
            let city_kana_text = document.get_element_by_id("popup_content_city_kana").unwrap();
            let google_map = document.get_element_by_id("popup_content_google_map").unwrap();
            let wikipedia = document.get_element_by_id("popup_content_wikipedia").unwrap();
            city_text.set_inner_html(city_name);
            prefecture_text.set_inner_html(&RECORDS[i].prefecture);
            city_kana_text.set_inner_html(&RECORDS[i].city_kana);
            google_map.set_attribute("src", format!("https://www.google.com/maps?output=embed&q={}{}", &RECORDS[i].prefecture, city_name).as_str()).unwrap();
            wikipedia.set_attribute("href", format!("https://ja.wikipedia.org/wiki/{}", RECORDS[i].city.replace("（", "_(")).replace("）", ")").as_str()).unwrap();
        }));
        label.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    wasm_bindgen_futures::spawn_local(remove_text_after(label, 20000));
}

async fn remove_text_after(text: Element, millis: u64) {
    Delay::new(Duration::from_millis(millis)).await.unwrap();
    text.remove();
}

async fn fetch() -> Result<Vec<CityName>, ()> {
    let host = "https://ddhr36ot0te3x.cloudfront.net";
    let url = Url::parse(format!("{}/city_names.csv", host).as_str()).unwrap();

    let client = reqwest_wasm::Client::new();
    let request = client
        .get(url)
        .build()
        .unwrap();

    let csv = client.execute(request).await.unwrap().text().await.unwrap();

    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut results = vec![];
    for record in reader.records() {
        let record = record.unwrap();
        let city = CityName {
            prefecture: String::from(&record[0]),
            city: String::from(&record[1]),
            prefecture_kana: String::from(&record[2]),
            city_kana: String::from(&record[3]),
            latitude: record[4].parse::<f64>().unwrap(),
            longitude: record[5].parse::<f64>().unwrap(),
        };

        results.push(city);
    }

    Ok(results)
}
