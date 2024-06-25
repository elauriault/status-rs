use std::error::Error;

use clap::{App, Arg};
use serde_derive::{Deserialize, Serialize};
// use serde_derive::Deserialize;
// use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectGroup {
    name: String,
    id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Project {
    name: String,
    id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Pipeline {
    id: u32,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectStatus {
    id: u32,
    name: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GroupStatus {
    name: String,
    id: u32,
    projects: Vec<ProjectStatus>,
}

struct Gitlab {
    url: String,
    token: String,
}

impl Gitlab {
    async fn get_group_id_by_name(
        &self,
        groups: Vec<&str>,
        // ) -> std::result::Result<Vec<ProjectGroup>, &'static str> {
    ) -> std::result::Result<Vec<ProjectGroup>, Box<dyn Error + Sync + Send>> {
        let mut t: Vec<ProjectGroup> = Vec::new();
        let mut count = groups.len();
        let mut page: u32 = 1;

        while count > 0 {
            let url = "/api/v4/groups?page=".to_owned()
                + &format!("{}", page).to_owned()
                + "&per_page=10&all=False";

            let res = self.get_from_gitlab(url.as_str()).await?;

            let g: Vec<ProjectGroup> = res.json().await?;

            if !g.is_empty() {
                for i in g {
                    for n in &groups {
                        if i.name == *n {
                            let j = i.clone();
                            t.push(j);
                            count -= 1;
                        }
                    }
                }
            } else {
                count = 0;
            }
            page += 1;
        }
        Ok(t)
    }

    async fn get_from_gitlab(
        &self,
        path: &str,
    ) -> std::result::Result<reqwest::Response, Box<dyn Error + Sync + Send>> {
        let client = reqwest::Client::new();
        let url = self.url.to_owned() + path;

        let res = client
            .get(url.as_str())
            .header("Private-Token", self.token.as_str())
            .send()
            .await?;

        Ok(res)
    }

    async fn get_project_ids_for_group(
        &self,
        group: u32,
    ) -> std::result::Result<Vec<Project>, Box<dyn Error + Sync + Send>> {
        let mut page_id: u32 = 1;
        let mut res: Vec<Project> = Vec::new();

        let url = "/api/v4/groups/".to_owned()
            + &format!("{}", group).to_owned()
            + "/projects/?page="
            + &format!("{}", page_id)
            + "&per_page=10&all=False";
        let page = self.get_from_gitlab(url.as_str()).await?;

        let mut page_json: Vec<Project> = page.json().await?;

        res.append(&mut page_json);

        while !page_json.is_empty() {
            page_id += 1;

            let page = self.get_from_gitlab(url.as_str()).await?;

            let mut page_json: Vec<Project> = page.json().await?;

            res.append(&mut page_json);
        }

        Ok(res)
    }

    async fn get_pipeline(
        &self,
        project: u32,
    ) -> std::result::Result<Pipeline, Box<dyn Error + Sync + Send>> {
        let url = "/api/v4/projects/".to_owned()
            + &format!("{}", project).to_owned()
            + "/pipelines/?page=1&per_page=1&all=False";

        let page = self.get_from_gitlab(url.as_str()).await?;

        let page_json: Vec<Pipeline> = page.json().await?;

        let res = page_json[0].clone();

        Ok(res)
    }

    async fn get_pipeline_status_by_group(
        &self,
        groups: Vec<ProjectGroup>,
    ) -> std::result::Result<Vec<GroupStatus>, Box<dyn Error + Sync + Send>> {
        let mut res: Vec<GroupStatus> = Vec::new();

        for group in groups {
            let mut g = GroupStatus {
                name: group.name,
                id: group.id,
                projects: Vec::new(),
            };
            let t = self.get_project_ids_for_group(group.id);
            let a = t.await?;
            // println!("{}", group.name);
            for r in a {
                let t = self.get_pipeline(r.id);
                let j = t.await?;
                let p = ProjectStatus {
                    id: r.id,
                    name: r.name,
                    status: j.status,
                };
                g.projects.push(p)
                // println!("{} : {:?}",r.name, j.status);
            }
            res.push(g);
        }

        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let matches = App::new("pipeline  status")
        .arg(
            Arg::with_name("gitlab")
                .help("gitlab url")
                .default_value("https://gitlab-ncsa.ubisoft.org/")
                .required(true)
                .takes_value(true)
                .short("g")
                .long("gitlab"),
        )
        .arg(
            Arg::with_name("token")
                .help("private token")
                .takes_value(true)
                .required(true)
                .short("t")
                .long("token"),
        )
        .arg(
            Arg::with_name("groups")
                .help("gitlab project groups")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let gitlab = matches.value_of("gitlab").unwrap();
    let token = matches.value_of("token").unwrap();
    let groups: Vec<&str> = matches.values_of("groups").unwrap().collect();
    // let mut count = groups.len();

    // println!("Value for gitlab: {:?}", gitlab);
    // println!("Value for token: {:?}", token);
    // println!("Value for groups: {:?}, it contains {} elements", groups, count);
    let g = Gitlab {
        url: gitlab.to_owned(),
        token: token.to_owned(),
    };

    let t = g.get_group_id_by_name(groups).await.unwrap();
    //.expect("Panicking!");
    let u = g.get_pipeline_status_by_group(t);
    // println!("{:?}", u.await.unwrap());
    println!("{}", serde_json::to_string(&u.await.unwrap()).unwrap());

    Ok(())
}
