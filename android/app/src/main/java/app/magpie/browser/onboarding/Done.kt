package app.magpie.browser.onboarding

const val ONBOARDING_DONE = "onboarding_done"

fun onboarded(saved: Map<String, *>): Boolean = saved[ONBOARDING_DONE] as? Boolean ?: saved.isNotEmpty()
