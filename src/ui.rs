use egg_mode::entities::{MediaEntity, UrlEntity};
use termion::{color, style};

pub(crate) struct ColorConfig {
    color_user: String,
    color_time: String,
    color_url: String,
    color_meta: String,
    color_hash: String,
}
impl ColorConfig {
    fn colorize(color: &str, s: &str) -> String {
        format!("{}{}{}", &color, s, style::Reset)
    }
    fn user(&self, s: &str) -> String {
        Self::colorize(&self.color_user, s)
    }
    fn time(&self, s: &str) -> String {
        Self::colorize(&self.color_time, s)
    }
    fn url(&self, s: &str) -> String {
        Self::colorize(&self.color_url, s)
    }
    fn meta(&self, s: &str) -> String {
        Self::colorize(&self.color_meta, s)
    }
    fn hash(&self, s: &str) -> String {
        Self::colorize(&self.color_hash, s)
    }
}

impl Default for ColorConfig {
    fn default() -> ColorConfig {
        ColorConfig {
            color_user: format!("{}{}", style::Bold, color::Fg(color::Blue)),
            color_time: format!("{}", color::Fg(color::Yellow)),
            color_url: format!("{}{}", style::Underline, color::Fg(color::Blue)),
            color_meta: format!("{}", color::Fg(color::Red)),
            color_hash: format!("{}", color::Fg(color::Yellow)),
        }
    }
}

pub(crate) struct UI {
    color_config: ColorConfig,
}
impl UI {
    pub fn new() -> Self {
        UI {
            color_config: ColorConfig::default(),
        }
    }

    // Preferred format:
    // @handle name at time
    // ♺:numrts ♥:numhearts via source from place
    // ➜ in reply to @handle OR
    // ➜ RT @handle
    //   tweet text indented
    // ➜ QT @handle
    //   quoted text indented
    // (blank line)
    // TODO: break this up into smaller pieces, please
    pub fn format_tweet(&self, tweet: &egg_mode::tweet::Tweet) -> String {
        let name: String = tweet
            .user
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or("<unknown>".to_string());
        let handle: String = tweet
            .user
            .as_ref()
            .map(|t| t.screen_name.clone())
            .unwrap_or("".to_string());
        let time: String = format!("{}", tweet.created_at.with_timezone(&chrono::Local));
        // .format("%l:%N%P")
        // .to_string();
        let header: String = format!(
            "@{} {} at {} ",
            self.color_config.user(&handle),
            &name,
            self.color_config.time(&time)
        );

        let via: String = tweet
            .source
            .as_ref()
            .map(|s| format!(" via {}", &s.name))
            .unwrap_or("".to_string());
        let from: String = tweet
            .place
            .as_ref()
            .map(|p| format!(" from {}", &p.full_name))
            .unwrap_or("".to_string());
        let meta: String = format!(
            "{}:{} {}:{}{}{}\n",
            self.color_config.meta("♺"),
            &tweet.retweet_count,
            self.color_config.meta("♥"),
            &tweet.favorite_count,
            &via,
            &from
        );

        let context = tweet
            .retweeted_status
            .as_ref()
            .map(|rt| {
                format!(
                    "{} @{} {} {}:{}\n",
                    self.color_config.meta("➜ RT"),
                    self.color_config
                        .user(&rt.user.as_ref().unwrap().screen_name),
                    &rt.user.as_ref().unwrap().name,
                    self.color_config.meta("♥"),
                    &rt.favorite_count,
                )
            })
            .or_else(|| {
                tweet.in_reply_to_screen_name.as_ref().map(|name| {
                    format!(
                        "{} {}{}{}{}\n",
                        self.color_config.meta("➜ In reply to"),
                        // TODO: if in_reply_to_status_id is None then need to return empty url
                        self.color_config.url("https://twitter.com/"),
                        self.color_config.url(name),
                        self.color_config.url("/status/"),
                        self.color_config
                            .url(&tweet.in_reply_to_status_id.unwrap_or(0).to_string()),
                    )
                })
            })
            .unwrap_or("".to_string());

        // TODO: don't lose the spaces or newlines between words
        // TODO: make simpler supporting functions. For example for name display.
        // TODO: merge line two up to line one and ditch via url
        // TODO: single blank between tweets. Make sure \n\n* becomes just \n

        let tweet: String = if let Some(ref rt) = tweet.retweeted_status {
            format!(
                "{}\n",
                self.colorize_tweet_text(&rt.text, &rt.entities.urls, &rt.entities.media)
            )
        } else if let Some(ref qt) = tweet.quoted_status {
            format!(
                "{}\n--\n{} {} {}\n{}\n",
                self.colorize_tweet_text(&tweet.text, &tweet.entities.urls, &tweet.entities.media),
                self.color_config.meta("➜ QT"),
                self.color_config
                    .user(&qt.user.as_ref().unwrap().screen_name),
                &qt.user.as_ref().unwrap().name,
                self.colorize_tweet_text(&qt.text, &qt.entities.urls, &qt.entities.media)
            )
        } else {
            format!(
                "{}\n",
                self.colorize_tweet_text(&tweet.text, &tweet.entities.urls, &tweet.entities.media)
            )
        };

        header + &meta + &context + &tweet //+ blankline
    }

    fn colorize_tweet_text(
        &self,
        text: &str,
        url_entities: &Vec<UrlEntity>,
        opt_media_entities: &Option<Vec<MediaEntity>>,
    ) -> String {
        let mut colored_tweet = String::new();
        for word in text.split_whitespace() {
            if word.starts_with("@") {
                colored_tweet.push_str(&self.color_config.user(word));
            } else if word.starts_with("#") {
                colored_tweet.push_str(&self.color_config.hash(word));
            } else if word.starts_with("http:") || word.starts_with("https:") {
                let url = url_entities
                    .iter()
                    .find_map(|url_entity| {
                        if url_entity.url == word {
                            url_entity.expanded_url.clone()
                        } else if url_entity.display_url == word {
                            url_entity.expanded_url.clone()
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        flatten_options(opt_media_entities.as_ref().map(|media_entities| {
                            media_entities.iter().find_map(|media_entity| {
                                if media_entity.url == word {
                                    Some(media_entity.media_url_https.clone())
                                } else if media_entity.display_url == word {
                                    Some(media_entity.media_url_https.clone())
                                } else {
                                    None
                                }
                            })
                        }))
                    })
                    .unwrap_or_else(|| word.to_string());
                colored_tweet.push_str(&self.color_config.url(&url));
            } else if word.contains("&amp;") {
                colored_tweet.push_str(&word.replace("&amp;", "&"));
            } else {
                colored_tweet.push_str(word);
            }
            colored_tweet.push_str(" ");
        }
        colored_tweet
    }

    // Cloudera @cloudera 30 seconds ago
    // ♺:1 ♥:0 id:2 via Sprout Social
    // "The healthcare sector can turn to #IoT, big data and #MachineLearning to power research, develop new treatments and improve existing ones — and it can also take the pain out of logistical failures." https://bit.ly/3xkcMxP
    //
    // NPR's Planet Money @planetmoney 38 seconds ago
    // ♺:3 ♥:0 id:7 via Twitter for iPhone
    // RT @adam_tooze: Shrinkflation: US consumers branding is truly DEVIOUS. If consumers don’t like price increases why not simply reduce the quantity? I suspect something similar with calorie counts. Crazy variety of units used, makes comparison into arithmetic test.
    // https://t.co/csJgJwIilx https://t.co/aezsT9Ts81
    // Color the metadata and also color the tags, links, and usernames in the tweets
    pub(crate) async fn print_tweet(&self, tweet: &egg_mode::tweet::Tweet) {
        println!("{}", &self.format_tweet(&tweet));
        // if let Some(ref user) = tweet.user {
        // println!(
        // "{}{} @{}{} posted at {}{}",
        // &self.color_config.color_user,
        // &user.name,
        // &user.screen_name,
        // style::Reset,
        // tweet.created_at.with_timezone(&chrono::Local),
        // style::Reset,
        // );
        // }

        // if let Some(ref screen_name) = tweet.in_reply_to_screen_name {
        // println!(
        // "➜ in reply to {}@{}{}",
        // self.color_config.color_user,
        // screen_name,
        // style::Reset
        // );
        // }

        // if let Some(ref status) = tweet.retweeted_status {
        // println!("RT @{} ➜", status.user.as_ref().unwrap().name);
        // println!("{}", &status.text);
        // return;
        // } else {
        // println!("{}", &tweet.text);
        // }

        // if let Some(source) = &tweet.source {
        // println!("➜ via {} ({})", source.name, source.url);
        // }

        // if let Some(ref place) = tweet.place {
        // println!("➜ from: {}", place.full_name);
        // }

        // if let Some(ref status) = tweet.quoted_status {
        // println!("{}", &tweet.text);
        // println!("QT @{} ➜", &status.user.as_ref().unwrap().name);
        // println!("{}", &status.text);
        // return;
        // }

        // if !tweet.entities.hashtags.is_empty() {
        // println!("➜ Hashtags contained in the tweet:");
        // for tag in &tweet.entities.hashtags {
        // println!("  {}", tag.text);
        // }
        // }

        // if !tweet.entities.symbols.is_empty() {
        // println!("➜ Symbols contained in the tweet:");
        // for tag in &tweet.entities.symbols {
        // println!("  {}", tag.text);
        // }
        // }

        // if !tweet.entities.urls.is_empty() {
        // println!("➜ URLs contained in the tweet:");
        // for url in &tweet.entities.urls {
        // if let Some(expanded_url) = &url.expanded_url {
        // println!("  {}", expanded_url);
        // }
        // }
        // }

        // if !tweet.entities.user_mentions.is_empty() {
        // println!("➜ Users mentioned in the tweet:");
        // for user in &tweet.entities.user_mentions {
        // println!("  {}", Paint::bold(Paint::blue(&user.screen_name)));
        // }
        // }

        // if let Some(ref media) = tweet.extended_entities {
        // println!("➜ Media attached to the tweet:");
        // for info in &media.media {
        // println!("  A {:?}", info.media_type);
        // }
        // }
        //println!("");
    }
}

fn flatten_options<T>(oot: Option<Option<T>>) -> Option<T> {
    match oot {
        None => None,
        Some(v) => v,
    }
}
