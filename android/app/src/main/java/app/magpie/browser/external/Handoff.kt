package app.magpie.browser.external

enum class Handoff { Open, Confirm }

data class Navigation(val userGesture: Boolean, val typed: Boolean)

fun handoff(navigation: Navigation): Handoff =
    if (navigation.userGesture || navigation.typed) Handoff.Open else Handoff.Confirm
