use hashbrown::HashSet;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};

use crate::{
    magnet::{ExtractMagnetContextErr, Magnet, MagnetContext},
    wait::Waiter,
};

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:105.0) Gecko/20100101 Firefox/105.0";

trait LinkBuilder {
    fn link(&self, page: usize) -> String;
}

struct SearchLinkBuilder {
    query: String,
}

impl LinkBuilder for SearchLinkBuilder {
    fn link(&self, page: usize) -> String {
        // https://thepiratebay10.org/search/James%20Deen/1/3/0

        // The path segment "3" refers to sorting by upload date.
        // The last path segment refers to the search category.

        let query = &self.query;
        format!("https://thepiratebay10.org/search/{query}/{page}/3/0")
    }
}

struct UserLinkBuilder {
    user: String,
}

impl LinkBuilder for UserLinkBuilder {
    fn link(&self, page: usize) -> String {
        // https://thepiratebay10.org/user/PornBaker/2/3

        // Note the absence of the search category from above.

        let user = &self.user;
        format!("https://thepiratebay10.org/user/{user}/{page}/3")
    }
}

fn link_builder(url: &str) -> Option<Box<dyn LinkBuilder>> {
    let search = Regex::new(r#"/search/([^/]+)/"#).unwrap();
    if let Some(cx) = search.captures(url) {
        return cx.get(1).map(|cx| {
            Box::new(SearchLinkBuilder {
                query: cx.as_str().into(),
            }) as Box<dyn LinkBuilder>
        });
    }

    let user = Regex::new(r#"/user/([^/]+)/"#).unwrap();
    if let Some(cx) = user.captures(url) {
        return cx.get(1).map(|cx| {
            Box::new(UserLinkBuilder {
                user: cx.as_str().into(),
            }) as Box<dyn LinkBuilder>
        });
    }

    None
}

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
        limit: usize,
        filter: &mut HashSet<String>,
        waiter: &mut Waiter,
    ) -> anyhow::Result<Vec<Magnet>> {
        // We need to begin pagination with 1 or there's going to be weirdness.
        let pages = 1..=limit;
        let links = link_builder(url).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("unsupported url: {url}"),
            )
        })?;

        let page_links = pages.map(|page| links.link(page));

        let mut magnets = Vec::new();

        for url in page_links {
            waiter.wait();

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
