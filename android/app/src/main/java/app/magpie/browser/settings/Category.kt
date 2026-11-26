package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Extension
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.ui.graphics.vector.ImageVector
import app.magpie.browser.focus.Distraction

enum class Category(val label: String, val icon: ImageVector, private val terms: List<String>) {
    General(
        "General",
        Icons.Outlined.Layers,
        listOf("Swipe address bar to switch tabs", "Spaces", "Profiles", "New space", "Rename space", "Clear profile data", "Delete space", "Backup", "Export Magpie backup", "Import Magpie backup"),
    ),
    Appearance("Appearance", Icons.Outlined.Palette, listOf("Theme", "Menu side") + MenuSide.entries.map(MenuSide::placement)),
    Search("Search", Icons.Outlined.Search, listOf("Default search engine", "Custom search engine") + SearchEngine.entries.map(SearchEngine::label)),
    Privacy(
        "Privacy & Security",
        Icons.Outlined.Shield,
        listOf(
            "Ad blocker",
            "Strip tracking from URLs",
            "Tracking protection",
            "HTTPS-Only mode",
            "Ask sites not to track you",
            "Global Privacy Control",
            "Do Not Track",
            "Ad blocker paused on",
            "Site permissions",
            "Clear browsing data",
            "Clear browsing data on exit",
        ),
    ),
    Passwords("Passwords", Icons.Outlined.Key, listOf("Android System Autofill", "Change autofill service", "Built-in Bitwarden Vault", "Bitwarden", "Open vault")),
    Addons(
        "Add-ons",
        Icons.Outlined.Extension,
        listOf("SponsorBlock", "Show skip notice", "Skip these segments", "Paused channels", "Distraction Blocker") +
            Distraction.entries.map(Distraction::label) + listOf("Background play", "Reader mode", "Force dark mode"),
    ),
    About(
        "About",
        Icons.Outlined.Info,
        listOf("Version", "GeckoView", "Magpie core", "Check for updates", "Check for updates automatically", "Update channel", "Nightly", "Download & Install"),
    );

    fun matches(query: String): List<String> = (listOf(label) + terms).filter { it.contains(query, ignoreCase = true) }
}

data class Match(val category: Category, val term: String)

fun search(query: String): List<Match> =
    Category.entries.flatMap { category -> category.matches(query).map { Match(category, it) } }
