package app.magpie.browser.password

enum class Service(val overline: String, val label: String, val detail: String) {
    SystemAutofill(
        "Autofill service",
        "Android System Autofill",
        "Uses your phone’s password manager, such as Bitwarden, Google Password Manager or 1Password. Nothing is stored in Magpie.",
    ),
    BuiltIn("Optional", "Built-in Bitwarden Vault", "Magpie keeps your Bitwarden logins on this phone."),
}
