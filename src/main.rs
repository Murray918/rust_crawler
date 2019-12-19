use html5ever::tokenizer::{
    BufferQueue,
    Tag,
    TagKind,
    TagToken,
    Token,
    TokenSink,
    TokenSinkResult,
    Tokenizer,
    TokenizerOpts,
};

use std::borrow::Borrow;

use async_std::task;

use url::{
    ParseError, 
    Url,
};

type CrawlResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

type BoxFuture = std::pin::Pin<Box<dyn  std::future::Future<Output=CrawlResult> + Send>>;



#[derive(Default, Debug)]
struct LinkQueue {
    links: Vec<String>,
}

impl TokenSink for &mut LinkQueue {
    type Handle = ();

    fn process_token(
        &mut self,
        token: Token,
        line_number: u64
    ) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(
                ref
                tag
                @
                Tag {
                    kind: TagKind::StartTag,
                    ..
                },
            ) => {
                if tag.name.as_ref() == "a" {
                    for attribute in tag.attrs.iter() {
                        if attribute.name.local.as_ref() == "href" {
                            let url_str: &[u8] = attribute.value.borrow();
                            self.links
                                .push(String::from_utf8_lossy(url_str).into_owned());
                        }
                    }
                }
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

pub fn get_links(url: &Url, page: String) -> Vec<Url> {
    let mut domain_url = url.clone();
    domain_url.set_path("");
    domain_url.set_query(None);

    // 0 out the links vector
    let mut queue = LinkQueue::default(); 
    //identify tokens in the html page
    let mut tokenizer = Tokenizer::new(&mut queue, TokenizerOpts::default());
    //reading through the characters of our html page
    let mut buffer = BufferQueue::new(); 

    buffer.push_back(page.into());
    let _ = tokenizer.feed(&mut buffer);

    queue
        .links
        .iter()
        .map(|link| match Url::parse(link){
            //if the url is relative join it with the domain clone it an push it in to the links
            Err(ParseError::RelativeUrlWithoutBase) => domain_url.join(link).unwrap(),
            //if it is not a link panic
            Err(_) => panic!("Malformed Link found {}", link),
            //return the link into links
            Ok(url) => url
        }) 
    .collect() // collect our info
}   

async fn crawl (pages : Vec<Url>, current: u8, max: u8 ) -> CrawlResult {
    println!("Current Depth: {}, Max Depth: {}", current, max);

    if current > max {
        println!("Reached Max Depth");
        return Ok(());
    }
    
    let mut tasks = vec![]
    
    println!("crawling: {:?}", pages);

    for url in pages {
        let task = task::spawn(async move {
            println!("Getting : {}", url);

            let mut res = surf::get(&url).await?;
            let body = res.body_string().await?;

            let links = get_links(&url, body);
            
            println!("Following: {:?}", links);
        });

        tasks.push(task)
    }

    for task in tasks.into_iter() {
        task.await?;
    }

    Ok(())
}

fn main() {
    println!("Hello, world!");
}
