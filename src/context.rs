use hashbrown::HashSet;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};

use crate::magnet::{ExtractMagnetContextErr, Magnet, MagnetContext};

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:99.0) Gecko/20100101 Firefox/99.0";

pub struct Context {
    client: Client,
    det_selector: Selector,
    page_link_selector: Selector,
    magnet_link_selector: Selector,
    info_selector: Selector,
    size_pattern: Regex,
}

impl Context {
    pub fn new() -> Self {
        Context {
            client: build_client(),
            det_selector: Selector::parse("td > div.detName").unwrap(),
            page_link_selector: Selector::parse("div.detName > a").unwrap(),
            magnet_link_selector: Selector::parse("div.detName + a").unwrap(),
            info_selector: Selector::parse("font").unwrap(),
            size_pattern: Regex::new(r#"Size ([\d.]+)&nbsp;([^,]+)"#).unwrap(),
        }
    }

    pub fn extract_recent(
        &self,
        url: &str,
        filter: &mut HashSet<String>,
    ) -> anyhow::Result<Vec<Magnet>> {
        let mut magnets = Vec::new();
        let text = self.client.get(url).send()?.text()?;
        let document = Html::parse_fragment(&text);
        let det_elements = document
            .select(&self.det_selector)
            .filter_map(|element| ElementRef::wrap(element.parent()?));

        for element in det_elements {
            let link = self.get_magnet_link(&element)?;
            if filter.insert(link.to_string()) {
                let info = self.get_info(&element)?;
                let size = self
                    .size_pattern
                    .captures(&info)
                    .ok_or_else(|| ExtractMagnetContextErr::Size(info.to_string()))?;

                let magnet_context = MagnetContext {
                    text: self.get_link_text(&element)?,
                    link: self.get_magnet_link(&element)?,
                    size: format!(
                        "{} {}",
                        size.get(1).unwrap().as_str(),
                        size.get(2).unwrap().as_str()
                    ),
                    info: self.get_info(&element)?,
                };

                magnets.push(magnet_context.try_into()?);
            }
        }

        Ok(magnets)
    }

    fn get_link_text(&self, element: &ElementRef) -> Result<String, ExtractMagnetContextErr> {
        let link_element = element
            .select(&self.page_link_selector)
            .next()
            .ok_or_else(|| ExtractMagnetContextErr::PageLink(element.html()))?;
        Ok(link_element.inner_html())
    }

    fn get_magnet_link<'a>(
        &self,
        element: &'a ElementRef,
    ) -> Result<&'a str, ExtractMagnetContextErr> {
        let link_element = element
            .select(&self.magnet_link_selector)
            .next()
            .and_then(|element| element.value().attr("href"))
            .ok_or_else(|| ExtractMagnetContextErr::MagnetLink(element.html()))?;
        Ok(link_element)
    }

    fn get_info(&self, element: &ElementRef) -> Result<String, ExtractMagnetContextErr> {
        let info_element = element
            .select(&self.info_selector)
            .next()
            .ok_or_else(|| ExtractMagnetContextErr::Info(element.html()))?;
        Ok(info_element.inner_html())
    }
}

fn build_client() -> Client {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("unreachable: client")
}

#[cfg(test)]
mod tests {
    #[test]
    fn build_client() {
        super::build_client();
    }
}
