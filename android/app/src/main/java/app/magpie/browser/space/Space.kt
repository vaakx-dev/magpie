package app.magpie.browser.space

sealed interface Profile {
    val label: String

    data class Named(val id: String, override val label: String) : Profile

    data object Private : Profile {
        override val label = "Private · nothing saved"
    }
}

data class Space(val id: String, val name: String, val profile: Profile) {
    companion object {
        val PRIVATE = Space("private", "Private", Profile.Private)
    }
}
