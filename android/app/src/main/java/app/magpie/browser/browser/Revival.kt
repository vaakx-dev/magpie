package app.magpie.browser.browser

const val CRASH_LOOP_MS = 15_000L

enum class Presence { Shown, Hidden }

enum class Revive { Now, WhenShown }

class Revival {
    private var lastEndMs: Long? = null

    fun ended(atMs: Long, presence: Presence): Revive {
        val looping = lastEndMs?.let { atMs - it < CRASH_LOOP_MS } == true
        lastEndMs = atMs
        return if (presence == Presence.Shown && !looping) Revive.Now else Revive.WhenShown
    }
}
