package app.magpie.browser.update

private val NIGHTLY_VERSION = Regex("""(.+)-nightly\.(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})""")

fun versionLabel(version: String, commit: String): String {
    val nightly = NIGHTLY_VERSION.matchEntire(version) ?: return "Magpie $version"
    val (base, year, month, day, hour, minute) = nightly.destructured
    val build = listOf("$year-$month-$day $hour:$minute UTC", commit).filter { it.isNotEmpty() }.joinToString(" · ")
    return "Magpie $base (Nightly $build)"
}
