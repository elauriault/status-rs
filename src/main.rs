extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate reqwest;
extern crate clap;
extern crate tokio;

use clap::{Arg, App};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct ProjectGroup {
    name: String,
    id: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct Project {
    name: String,
    id: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct Pipeline {
    id: u32,
    status: String,
}

struct Gitlab {
    url: String,
    token: String,
}

impl Gitlab {

    async fn get_group_id_by_name(&self, groups: Vec<&str>) -> std::result::Result<Vec<ProjectGroup>, &'static str>{

        let mut t : Vec<ProjectGroup> = Vec::new();
        let mut count = groups.len();
        let mut page: u32 = 1;

        while count > 0{
            let url = "/api/v4/groups?page=".to_owned() + &format!("{}", page).to_owned() + "&per_page=10&all=False" ; 

            let res = self.get_from_gitlab(url.as_str())
                .await?;

            let g : Vec<ProjectGroup> = res.json()
                .await
                .expect("Panic!");

            if g.len() > 0 {
                for i in g{
                    for n in &groups{
                        if i.name == n.to_owned(){
                            let j = i.clone();
                            t.push(j);
                            count-=1;
                        }
                    }
                }
            } else {
                count=0;
            }
            page+=1;
        }
        Ok(t)

    }


    async fn get_from_gitlab(&self, path: &str) -> std::result::Result<reqwest::Response, &'static str>{

        let client = reqwest::Client::new();
        let url = self.url.to_owned() + path ; 

        let res = client
            .get(url.as_str())
            .header("Private-Token", self.token.as_str())
            .send()
            .await
            .expect("Panicking!");

        Ok(res)
    }

    async fn get_project_ids_for_group(&self, group: u32) -> std::result::Result<Vec<Project>, &'static str>{

        let mut page_id: u32 = 1;
        let mut res : Vec<Project> = Vec::new();

        let url = "/api/v4/groups/".to_owned() + &format!("{}", group).to_owned() + "/projects/?page=" + &format!("{}", page_id) + "&per_page=10&all=False" ; 
        let page = self.get_from_gitlab(&url.as_str() )
            .await?;

        let mut page_json : Vec<Project> = page.json()
            .await
            .expect("Panicking!");

        res.append(& mut page_json);

        while page_json.len() > 0 {
            page_id += 1;

            let page = self.get_from_gitlab(&url.as_str() )
                .await?;

            let mut page_json : Vec<Project> = page.json()
                .await
                .expect("Panicking!");

            res.append(& mut page_json);
        }

        Ok(res)

    }

    async fn get_pipeline(&self, project: u32) -> std::result::Result<Pipeline, &'static str>{

        let url = "/api/v4/projects/".to_owned() + &format!("{}", project).to_owned() + "/pipelines/?page=1&per_page=1&all=False" ; 

        let page = self.get_from_gitlab(&url.as_str())
            .await?;

        let page_json : Vec<Pipeline> = page.json()
            .await
            .expect("Panicking");

        let res = page_json[0].clone();

        Ok(res)

    }

    async fn get_pipeline_status_by_group(&self, groups: Vec<ProjectGroup>) -> std::result::Result<Vec<Project>, &'static str>{

        let res : Vec<Project> = Vec::new();

        for group in groups{
            let t = self.get_project_ids_for_group(group.id);
            let a = t.await?;
            println!("{}", group.name);
            for r in a{
                let t = self.get_pipeline(r.id);
                let j = t.await?;
                println!("{} : {:?}",r.name, j.status);

            }
        }

        Ok(res)

    }

}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {

    let matches = App::new("pipeline  status")
        .arg(Arg::with_name("gitlab")
                .help("gitlab url")
                .default_value("https://gitlab-ncsa.ubisoft.org/")
                .required(true)
                .takes_value(true)
                .short("g")
                .long("gitlab"))
         .arg(Arg::with_name("token")
                .help("private token")
                .takes_value(true)
                .required(true)
                .short("t")
                .long("token"))
         .arg(Arg::with_name("groups")
                .help("gitlab project groups")
                .takes_value(true)
                .multiple(true))
         .get_matches();

    let gitlab = matches.value_of("gitlab").unwrap();
    let token = matches.value_of("token").unwrap();
    let groups: Vec<&str> = matches.values_of("groups").unwrap().collect();
    // let mut count = groups.len();

    println!("Value for gitlab: {:?}", gitlab);
    println!("Value for token: {:?}", token);
    // println!("Value for groups: {:?}, it contains {} elements", groups, count);
    let g = Gitlab {
        url : gitlab.to_owned(),
        token: token.to_owned(),
    };

    let t = g.get_group_id_by_name(groups)
        .await
        .unwrap();
        //.expect("Panicking!");
    let u = g.get_pipeline_status_by_group(t);
    println!("{:?}", u.await);

    Ok(())

}
