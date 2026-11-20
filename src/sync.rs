//! The tool crib on the EH3D server: the table of the tools is kept as a project of the server, one revision for each time it is pushed, so that another computer (or the
//! same one after a mistake) can read the last one back, and the history can be listed. The crib travels as an artifact (`application/json`) and each revision of the
//! project points to one artifact, through the routes `/api/v1/artifacts` and `/api/v1/projects` of the server; none of them needs a header, and a revision says which
//! revision it follows, so two computers that push at the same time do not overwrite each other: the second one is refused and has to pull first.

use crate::server::{explain, send_artifact};
use eh3d_serverclient::json::{quote, Json};
use eh3d_serverclient::{path_segment, Client, Lang};

/// The name of the project of the crib when none is given.
pub const DEFAULT_NAME: &str = "tool-crib";
/// The longest name of a project the server takes.
pub const MAX_NAME: usize = 80;

/// What a push did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pushed {
    pub project_id: String,
    pub revision: u64,
    pub artifact_id: String,
    /// The project was made by this push.
    pub created: bool,
}

/// What a pull found: the last revision of the project and the text of its crib.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pulled {
    pub project_id: String,
    pub revision: u64,
    pub artifact_id: String,
    pub text: String,
}

/// One revision of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    pub number: u64,
    pub artifact_id: String,
    pub by: String,
}

/// The name of a project as the server takes it: 1 to 80 characters, no control character.
pub fn check_name(name: &str) -> Result<&str, String> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > MAX_NAME || n.chars().any(char::is_control) {
        return Err(format!(
            "'{name}' cannot be the name of a project (1 to {MAX_NAME} characters, no control character)"
        ));
    }
    Ok(n)
}

fn parse(body: &str) -> Result<Json, String> {
    Json::parse(body).map_err(|e| format!("the answer of the server is not JSON: {e}"))
}

/// The id and the head revision of the live project with this name, when there is one.
fn find_project(
    client: &mut Client,
    name: &str,
    lang: Lang,
) -> Result<Option<(String, u64)>, String> {
    let reply = client
        .get("/api/v1/projects")
        .map_err(|e| explain(&e, lang))?;
    let list = parse(&reply.body)?;
    let items = list
        .as_array()
        .ok_or_else(|| "the list of projects of the server is not a list".to_string())?;
    for p in items {
        let same = p.get("name").and_then(Json::as_str) == Some(name);
        let gone = p.get("deleted").and_then(Json::as_bool) == Some(true);
        if same && !gone {
            let id = p.get("id").and_then(Json::as_str).unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let head = p.get("head").and_then(Json::as_u64).unwrap_or(0);
            return Ok(Some((id.to_string(), head)));
        }
    }
    Ok(None)
}

/// Keeps the crib (`data`, the text of its JSON) as a new revision of the project `name`, making the project the first time. The artifact is made first; a refusal of
/// the revision (the project moved on since the last pull) leaves the artifact on the server, where it is harmless.
pub fn push(client: &mut Client, name: &str, data: &[u8], lang: Lang) -> Result<Pushed, String> {
    let name = check_name(name)?;
    let sent = send_artifact(
        client,
        &format!("{name}.json"),
        "application/json",
        data,
        &[("kind", "tool-crib")],
        lang,
    )?;
    let (project_id, head, created) = match find_project(client, name, lang)? {
        Some((id, head)) => (id, head, false),
        None => {
            let reply = client
                .post(
                    "/api/v1/projects",
                    Some(&format!("{{\"name\":{}}}", quote(name))),
                )
                .map_err(|e| explain(&e, lang))?;
            let made = parse(&reply.body)?;
            let id = made
                .get("id")
                .and_then(Json::as_str)
                .ok_or("the answer of the server has no id of the project")?;
            (id.to_string(), 0, true)
        }
    };
    let reply = client
        .post(
            &format!("/api/v1/projects/{}/revisions", path_segment(&project_id)),
            Some(&format!(
                "{{\"artifact_id\":{},\"parent\":{head}}}",
                quote(&sent.artifact_id)
            )),
        )
        .map_err(|e| format!("{}: not added as a revision: {}", name, explain(&e, lang)))?;
    let revision = parse(&reply.body)?
        .get("number")
        .and_then(Json::as_u64)
        .unwrap_or(head + 1);
    Ok(Pushed {
        project_id,
        revision,
        artifact_id: sent.artifact_id,
        created,
    })
}

/// The id of the project and every revision of it, the oldest first.
fn project_revisions(
    client: &mut Client,
    name: &str,
    lang: Lang,
) -> Result<(String, Vec<Revision>), String> {
    let name = check_name(name)?;
    let (id, _) = find_project(client, name, lang)?
        .ok_or_else(|| format!("the server has no project named '{name}'"))?;
    let reply = client
        .get(&format!("/api/v1/projects/{}", path_segment(&id)))
        .map_err(|e| explain(&e, lang))?;
    let project = parse(&reply.body)?;
    let mut out = Vec::new();
    for r in project
        .get("revisions")
        .and_then(Json::as_array)
        .unwrap_or(&[])
    {
        let artifact_id = r.get("artifact_id").and_then(Json::as_str).unwrap_or("");
        if artifact_id.is_empty() {
            continue;
        }
        out.push(Revision {
            number: r.get("number").and_then(Json::as_u64).unwrap_or(0),
            artifact_id: artifact_id.to_string(),
            by: r.get("by").and_then(Json::as_str).unwrap_or("").to_string(),
        });
    }
    out.sort_by_key(|r| r.number);
    Ok((id, out))
}

/// Every revision of the project, the oldest first.
pub fn history(client: &mut Client, name: &str, lang: Lang) -> Result<Vec<Revision>, String> {
    Ok(project_revisions(client, name, lang)?.1)
}

/// The last revision of the project and the text of its crib.
pub fn pull(client: &mut Client, name: &str, lang: Lang) -> Result<Pulled, String> {
    let (project_id, revisions) = project_revisions(client, name, lang)?;
    let last = revisions
        .last()
        .ok_or_else(|| format!("the project '{}' has no revision yet", name.trim()))?;
    let reply = client
        .get(&format!(
            "/api/v1/artifacts/{}",
            path_segment(&last.artifact_id)
        ))
        .map_err(|e| explain(&e, lang))?;
    Ok(Pulled {
        project_id,
        revision: last.number,
        artifact_id: last.artifact_id.clone(),
        text: reply.body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eh3d_serverclient::{Reply, Request, Settings, Transport, TransportError};
    use std::sync::{Arc, Mutex};

    /// One revision of a project in the double: the artifact it points to.
    type Revisions = Vec<String>;

    #[derive(Default)]
    struct State {
        artifacts: Vec<(String, String)>,
        projects: Vec<(String, String, Revisions)>,
        requests: Vec<String>,
    }

    /// A server in memory that has the routes of the artifacts and of the projects, as the real one answers them.
    #[derive(Clone, Default)]
    struct Fake(Arc<Mutex<State>>);

    fn answer(status: u16, body: &str) -> Result<Reply, TransportError> {
        Ok(Reply {
            status,
            body: body.to_string(),
        })
    }

    impl Transport for Fake {
        fn send(&self, r: &Request) -> Result<Reply, TransportError> {
            let mut st = self.0.lock().unwrap();
            st.requests.push(format!("{} {}", r.method, r.path));
            let body = r.body.clone().unwrap_or_default();
            let json = Json::parse(&body).ok();
            let field = |k: &str| {
                json.as_ref()
                    .and_then(|j| j.get(k))
                    .and_then(Json::as_str)
                    .unwrap_or("")
                    .to_string()
            };
            let path = r.path.as_str();
            match (r.method.as_str(), path) {
                ("POST", "/api/v1/artifacts") => {
                    let data = field("data_base64");
                    let id = format!("a-{}", st.artifacts.len() + 1);
                    st.artifacts.push((id.clone(), data));
                    answer(
                        201,
                        &format!("{{\"artifact_id\":\"{id}\",\"blob_sha256\":\"ff\",\"size\":1}}"),
                    )
                }
                ("GET", "/api/v1/projects") => {
                    let rows: Vec<String> = st
                        .projects
                        .iter()
                        .map(|(id, name, revs)| {
                            format!(
                                "{{\"id\":\"{id}\",\"name\":{},\"owner\":\"u\",\"head\":{}}}",
                                quote(name),
                                revs.len()
                            )
                        })
                        .collect();
                    answer(200, &format!("[{}]", rows.join(",")))
                }
                ("POST", "/api/v1/projects") => {
                    let id = format!("p-{}", st.projects.len() + 1);
                    st.projects.push((id.clone(), field("name"), Vec::new()));
                    answer(
                        201,
                        &format!(
                            "{{\"id\":\"{id}\",\"name\":{},\"owner\":\"u\"}}",
                            quote(&field("name"))
                        ),
                    )
                }
                ("POST", p) if p.starts_with("/api/v1/projects/") && p.ends_with("/revisions") => {
                    let id = p["/api/v1/projects/".len()..p.len() - "/revisions".len()].to_string();
                    let artifact = field("artifact_id");
                    let parent = json
                        .as_ref()
                        .and_then(|j| j.get("parent"))
                        .and_then(Json::as_u64)
                        .unwrap_or(u64::MAX);
                    let Some(project) = st.projects.iter_mut().find(|p| p.0 == id) else {
                        return answer(404, "{\"error\":\"unknown_project\",\"detail\":\"\"}");
                    };
                    if parent != project.2.len() as u64 {
                        return answer(
                            409,
                            "{\"error\":\"revision_conflict\",\"detail\":\"the project moved on\"}",
                        );
                    }
                    project.2.push(artifact);
                    answer(201, &format!("{{\"number\":{}}}", project.2.len()))
                }
                ("GET", p) if p.starts_with("/api/v1/projects/") => {
                    let id = &p["/api/v1/projects/".len()..];
                    match st.projects.iter().find(|p| p.0 == id) {
                        Some((id, name, revs)) => {
                            let rows: Vec<String> = revs
                                .iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    format!(
                                        "{{\"number\":{},\"artifact_id\":\"{a}\",\"by\":\"u\"}}",
                                        i + 1
                                    )
                                })
                                .collect();
                            answer(
                                200,
                                &format!(
                                    "{{\"id\":\"{id}\",\"name\":{},\"owner\":\"u\",\"head\":{},\"revisions\":[{}]}}",
                                    quote(name),
                                    revs.len(),
                                    rows.join(",")
                                ),
                            )
                        }
                        None => answer(404, "{\"error\":\"unknown_project\",\"detail\":\"\"}"),
                    }
                }
                ("GET", p) if p.starts_with("/api/v1/artifacts/") => {
                    let id = &p["/api/v1/artifacts/".len()..];
                    match st.artifacts.iter().find(|a| a.0 == id) {
                        // the double keeps the base64 the program sent and gives the text back
                        Some((_, data)) => answer(200, &unbase64(data)),
                        None => answer(404, "{\"error\":\"unknown_artifact\",\"detail\":\"\"}"),
                    }
                }
                _ => answer(404, "{\"error\":\"not_found\",\"detail\":\"\"}"),
            }
        }
    }

    fn unbase64(text: &str) -> String {
        let value = |c: u8| match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        };
        let mut out = Vec::new();
        let mut bits = 0u32;
        let mut count = 0;
        for c in text.bytes() {
            if let Some(v) = value(c) {
                bits = (bits << 6) | u32::from(v);
                count += 6;
                if count >= 8 {
                    count -= 8;
                    out.push((bits >> count) as u8);
                    bits &= (1 << count) - 1;
                }
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn client(fake: &Fake) -> Client {
        let mut s = Settings::default();
        s.set("server_address", "127.0.0.1:18200").unwrap();
        s.set("server_token", "program-token").unwrap();
        Client::new(s, Box::new(fake.clone()))
    }

    #[test]
    fn the_first_push_makes_the_project_and_the_next_ones_add_revisions() {
        let fake = Fake::default();
        let mut c = client(&fake);
        let first = push(&mut c, "shop", b"{\"tools\":[]}", Lang::En).unwrap();
        assert!(first.created);
        assert_eq!((first.revision, first.project_id.as_str()), (1, "p-1"));
        let second = push(&mut c, "shop", b"{\"tools\":[1]}", Lang::En).unwrap();
        assert!(!second.created);
        assert_eq!((second.revision, second.project_id.as_str()), (2, "p-1"));
        assert_ne!(first.artifact_id, second.artifact_id);
        // another name is another project
        let other = push(&mut c, "bench", b"{}", Lang::En).unwrap();
        assert!(other.created && other.project_id == "p-2" && other.revision == 1);
    }

    #[test]
    fn a_pull_gives_back_the_last_revision_and_the_history_lists_them_all() {
        let fake = Fake::default();
        let mut c = client(&fake);
        push(&mut c, "shop", b"one", Lang::En).unwrap();
        push(&mut c, "shop", b"two \"quoted\"\nnew line", Lang::En).unwrap();
        let pulled = pull(&mut c, "shop", Lang::En).unwrap();
        assert_eq!(pulled.text, "two \"quoted\"\nnew line");
        assert_eq!(pulled.revision, 2);
        let h = history(&mut c, "shop", Lang::En).unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!((h[0].number, h[1].number), (1, 2));
        assert_eq!(h[1].artifact_id, pulled.artifact_id);
        assert_eq!(h[0].by, "u");
    }

    #[test]
    fn a_project_that_is_not_there_or_has_no_revision_is_told_in_words() {
        let fake = Fake::default();
        let mut c = client(&fake);
        let e = pull(&mut c, "nothing", Lang::En).unwrap_err();
        assert!(e.contains("no project named 'nothing'"), "{e}");
        let e = history(&mut c, "nothing", Lang::En).unwrap_err();
        assert!(e.contains("no project named"), "{e}");
        // a project made by someone else and still empty
        c.post("/api/v1/projects", Some("{\"name\":\"empty\"}"))
            .unwrap();
        let e = pull(&mut c, "empty", Lang::En).unwrap_err();
        assert!(e.contains("no revision yet"), "{e}");
    }

    #[test]
    fn a_project_that_moved_on_refuses_the_push_and_says_so() {
        let fake = Fake::default();
        let mut mine = client(&fake);
        let mut yours = client(&fake);
        push(&mut mine, "shop", b"one", Lang::En).unwrap();
        // the head is read when the push starts: a push that follows another one in between is the one the server refuses
        let id = find_project(&mut yours, "shop", Lang::En)
            .unwrap()
            .unwrap()
            .0;
        push(&mut mine, "shop", b"two", Lang::En).unwrap();
        let stale = yours
            .post(
                &format!("/api/v1/projects/{id}/revisions"),
                Some("{\"artifact_id\":\"a-1\",\"parent\":1}"),
            )
            .unwrap_err();
        assert_eq!(stale.status(), Some(409));
        assert_eq!(stale.code(), Some("revision_conflict"));
    }

    #[test]
    fn the_name_of_the_project_is_checked_before_anything_is_sent() {
        let fake = Fake::default();
        let mut c = client(&fake);
        for bad in ["", "   ", "a\u{7}b"] {
            assert!(push(&mut c, bad, b"{}", Lang::En).is_err(), "{bad:?}");
        }
        assert!(push(&mut c, &"x".repeat(81), b"{}", Lang::En).is_err());
        assert!(fake.0.lock().unwrap().requests.is_empty());
        assert_eq!(check_name("  shop ").unwrap(), "shop");
        assert!(check_name(&"x".repeat(80)).is_ok());
        assert_eq!(DEFAULT_NAME, "tool-crib");
    }

    #[test]
    fn a_server_without_a_token_is_not_asked_and_a_refusal_is_told_with_its_detail() {
        let fake = Fake::default();
        let mut s = Settings::default();
        s.set("server_address", "127.0.0.1:18200").unwrap();
        let mut c = Client::new(s, Box::new(fake.clone()));
        let e = push(&mut c, "shop", b"{}", Lang::En).unwrap_err();
        assert!(e.contains("server_token"), "{e}");
        assert!(fake.0.lock().unwrap().requests.is_empty());
    }
}
