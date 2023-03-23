use super::auth::JiraClient;
use super::SurrealAny;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkFields {
    pub issuetype: Type,
    pub priority: Priority,
    pub status: Status,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkType {
    pub inward: String,
    pub outward: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkInwardOutwardParent {
    pub fields: LinkFields,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub inward_issue: Option<LinkInwardOutwardParent>,
    pub outward_issue: Option<LinkInwardOutwardParent>,
    #[serde(alias = "type")]
    pub link_type: LinkType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatorReporter {
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Assignee {
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderedFields {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Components {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    key: String,
    name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Status {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Priority {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Type {
    pub name: String,
    pub subtask: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldAuthor {
    pub display_name: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentBody {
    pub author: FieldAuthor,
    pub created: String,
    pub rendered_body: String,
    pub updated: String,
    pub update_author: FieldAuthor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comments {
    pub comments: Vec<CommentBody>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub comments: Option<Comments>,
    pub components: Vec<Components>,
    pub creator: Option<CreatorReporter>,
    pub issuetype: Type,
    pub issuelinks: Vec<Links>,
    pub labels: Vec<String>,
    pub parent: Option<LinkInwardOutwardParent>,
    pub priority: Option<Priority>,
    pub project: Project,
    pub reporter: Option<CreatorReporter>,
    pub status: Status,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketData {
    pub fields: Fields,
    pub key: String,
    pub rendered_fields: RenderedFields,
}

impl TicketData {
    pub async fn save_ticket_comments_from_api(
        &self,
        db: &SurrealAny,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Comments> {
        let url = format!("/rest/api/3/issue/{}/comment?expand=renderedBody", self.key);
        let response = jira_client.get_from_jira_api(&url).await?;

        let comments: Comments = serde_json::from_str(response.as_str())
            .expect("unable to convert comments resp to slice");
        let _db_update: TicketData = db.update(("tickets", &self.key)).merge(&self).await?;
        return Ok(comments);
    }

    pub async fn get_comments(
        &self,
        db: &SurrealAny,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Comments> {
        let ticket: TicketData = db
            .select(("tickets", &self.key))
            .await
            .expect("Failed to get TicketData from DB in get_comments");
        match ticket.fields.comments {
            None => {
                info!("in get_comments none block");
                let c = self.save_ticket_comments_from_api(db, jira_client).await?;
                info!("get_comments none block response -- {:?}", c);
                Ok(c)
            }
            Some(c) => {
                info!("in get_comments some block");
                info!("get_comments some block response -- {:?}", c);
                Ok(c)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraTickets {
    pub start_at: Option<i32>,
    pub max_results: Option<i32>,
    pub total: Option<i32>,
    pub issues: Vec<TicketData>,
}

// TODO: handle pagination
impl JiraTickets {
    pub async fn new() -> anyhow::Result<Self> {
        let issues = Vec::new();
        Ok(Self {
            start_at: None,
            max_results: None,
            total: None,
            issues,
        })
    }

    pub async fn get_tickets_from_jira_api(
        &self,
        jira_auth: &JiraClient,
        url: &str,
    ) -> Result<String, reqwest::Error> {
        let headers = jira_auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(url).send().await?.text().await;

        return response;
    }
}
