use crate::models::account::Account;
use handlebars::Handlebars;
use mail_send::{self, mail_builder::MessageBuilder, SmtpClientBuilder};
use std::collections::HashMap;

pub fn build_message<'b>(
    from_name: &'b str,
    from_email: &'b str,
    account: &'b Account,
    url: &'b str,
) -> MessageBuilder<'b> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars
        .register_template_file("register_body", "emails/register_body.hbs")
        .unwrap();
    let mut data = HashMap::new();
    data.insert("url", url);
    let body = handlebars.render("register_body", &data).unwrap();

    handlebars
        .register_template_file("register_subject", "emails/register_subject.hbs")
        .unwrap();
    let data: HashMap<&str, &str> = HashMap::new();
    let subject = handlebars
        .render("register_subject", &data)
        .unwrap()
        .lines()
        .next() // first line only
        .unwrap()
        .to_string();

    MessageBuilder::new()
        .from((from_name, from_email))
        .to(account.email.as_str())
        .subject(subject)
        .text_body(body)
}

pub async fn send_email<'b>(smtp_host: &str, email: MessageBuilder<'b>) {
    println!("smtp {} to {:?}", smtp_host, email);
    SmtpClientBuilder::new(smtp_host, 25)
        .allow_invalid_certs()
        .implicit_tls(false)
        .connect()
        .await
        .unwrap()
        .send(email)
        .await
        .unwrap();
}
