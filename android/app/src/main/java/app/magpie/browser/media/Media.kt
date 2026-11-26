package app.magpie.browser.media

import org.mozilla.geckoview.MediaSession

data class Media(
    val controller: MediaSession,
    val playing: Boolean = false,
    val title: String = "",
    val artist: String = "",
)
