use pdao_config::Config;
use pdao_types::governance::subsquare::{SubSquareReferendum, SubSquareReferendumList};
use pdao_types::substrate::chain::Chain;

pub struct SubSquareClient {
    http_client: reqwest::Client,
}

impl SubSquareClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(
                    config.http.request_timeout_seconds,
                ))
                .build()?,
        })
    }

    pub async fn fetch_referendum(
        &self,
        chain: &Chain,
        index: u32,
    ) -> anyhow::Result<Option<SubSquareReferendum>> {
        let url = format!(
            "https://{}.subsquare.io/api/gov2/referendums/{index}?simple=false",
            chain.chain,
        );
        let response = self.http_client.get(url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let refererendum = response.json::<SubSquareReferendum>().await?;
        Ok(Some(refererendum))
    }

    pub async fn fetch_referenda(
        &self,
        chain: &Chain,
        page: u16,
        page_size: u16,
    ) -> anyhow::Result<SubSquareReferendumList> {
        let url = format!(
            "https://{}.subsquare.io/api/gov2/referendums?simple=false&page_size={page_size}&page={page}",
            chain.chain,
        );
        Ok(self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<SubSquareReferendumList>()
            .await?)
    }
}
