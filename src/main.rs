use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api-pub.bitfinex.com/v2/conf/pub:list:pair:exchange";
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        println!(
            "API isteği başarısız oldu. Durum kodu: {}",
            response.status()
        );
        return Ok(());
    }

    let text = response.text().await?;
    let data: Value = serde_json::from_str(&text)?;

    let mut coins: Vec<String> = Vec::new();
    if let Some(pairs) = data[0].as_array() {
        for pair in pairs {
            if let Some(pair_str) = pair.as_str() {
                if pair_str.ends_with("USD") && !pair_str.starts_with("USD") {
                    let coin = pair_str
                        .trim_end_matches("USD")
                        .replace(":", "")
                        .to_uppercase();
                    coins.push(format!("BITFINEX:{}USD", coin));
                }
            }
        }
    }

    if coins.is_empty() {
        println!("USD çiftleri bulunamadı. API yanıtını kontrol edin.");
        return Ok(());
    }

    coins.sort_by(|a, b| {
        let a_name = a.split(':').nth(1).unwrap_or("").trim_end_matches("USD");
        let b_name = b.split(':').nth(1).unwrap_or("").trim_end_matches("USD");

        let a_numeric = a_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        let b_numeric = b_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        if !a_numeric.is_empty() && !b_numeric.is_empty() {
            let a_num: u32 = a_numeric.parse().unwrap_or(0);
            let b_num: u32 = b_numeric.parse().unwrap_or(0);
            if a_num != b_num {
                return b_num.cmp(&a_num);
            }
        }

        if a_numeric.is_empty() != b_numeric.is_empty() {
            return if a_numeric.is_empty() {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        a_name.cmp(b_name)
    });

    let mut file = File::create("bitfinex_usd_markets.txt")?;
    for coin in &coins {
        writeln!(file, "{}", coin)?;
    }

    println!("Bitfinex USD spot piyasası coin listesi başarıyla oluşturuldu ve kaydedildi.");
    println!("Toplam {} adet USD çifti bulundu.", coins.len());

    Ok(())
}
