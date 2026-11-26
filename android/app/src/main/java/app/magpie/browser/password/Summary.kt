package app.magpie.browser.password

fun label(status: Status?): String = when (status?.lock) {
    null -> "Checking…"
    Lock.Unavailable -> "Unavailable"
    Lock.SignedOut -> "Signed out"
    Lock.Challenged -> "Waiting for two-step login"
    Lock.Locked -> "Locked"
    Lock.Unlocked -> "Unlocked"
}

fun summary(status: Status?): String = when (status?.lock) {
    Lock.Unavailable -> "Needs Magpie's core library, which isn't in this build"
    Lock.Locked, Lock.Unlocked -> "${label(status)} · ${status.email}"
    else -> label(status)
}
