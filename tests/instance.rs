use magpie::instance::{addresses, decode, encode};

#[test]
fn launch_arguments_become_the_addresses_to_open() {
    let arguments = [
        " https://example.com/a b ",
        "",
        "   ",
        "C:\\Users\\Someone\\page.html",
        "magpie://settings#import",
    ]
    .map(str::to_owned);

    assert_eq!(
        addresses(arguments),
        [
            "https://example.com/a b",
            "C:\\Users\\Someone\\page.html",
            "magpie://settings#import"
        ]
    );
}

#[test]
fn handed_off_addresses_arrive_exactly_as_sent() {
    let sent = addresses(["https://例え.jp/?q=\"quoted\"&x=1", "news"].map(str::to_owned));

    assert_eq!(decode(&encode(&sent)), Some(sent));
    assert_eq!(decode(&encode(&[])), Some(Vec::new()));
    assert_eq!(decode(b"{\"not\": \"a list\"}"), None);
}
