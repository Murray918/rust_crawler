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

use url::{
    ParseError, 
    Url,
};

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

    let mut queue = LinkQueue::default(); // 0 out the links vector
    //identify tokens in the html page
    let mut tokenizer = Tokenizer::new(&mut queue, TokenizerOpts::default());
    let mut buffer = BufferQueue::new(); //reading through the characters of our html page
    buffer.push_back(page.into());
    let _ = tokenizer.feed(&mut buffer);

    queue
        .links
        .iter()
        .map(|link| match Url::parse(link){
            Err(ParseError::RelativeUrlWithoutBase) => domain_url.join(link).unwrap(),
            Err(_) => panic!("Malformed Link found {}", link),
            Ok(url) => url
        }) 
    .collect()

}   


fn main() {
    println!("Hello, world!");
}
