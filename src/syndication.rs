// This is a shameless copy of https://github.com/tomshen/rust-syndication to have it working with recent versions of
// both RSS and atom crates

use std::str::FromStr;

pub enum Feed {
    Atom(atom_syndication::Feed),
    RSS(rss::Channel),
}

impl FromStr for Feed {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<atom_syndication::Feed>() {
            Ok(feed) => Ok(Feed::Atom(feed)),
            _ => match s.parse::<rss::Channel>() {
                Ok(channel) => Ok(Feed::RSS(channel)),
                _ => Err("Could not parse XML as Atom or RSS from input"),
            },
        }
    }
}

impl ToString for Feed {
    fn to_string(&self) -> String {
        match *self {
            Feed::Atom(ref atom_feed) => atom_feed.to_string(),
            Feed::RSS(ref rss_channel) => rss_channel.to_string(),
        }
    }
}
