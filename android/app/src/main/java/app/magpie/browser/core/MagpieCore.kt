package app.magpie.browser.core

object MagpieCore {
    val loaded: Boolean = try {
        System.loadLibrary("magpie_core")
        true
    } catch (missing: UnsatisfiedLinkError) {
        false
    }

    external fun cleanUrl(url: String, stripTracking: Boolean): String

    external fun focusScript(blocked: String): String

    external fun focusUpdate(blocked: String): String

    external fun focusRedirect(blocked: String, url: String): String?

    external fun cachedFilters(dataDir: String): ByteArray?

    external fun refreshedFilters(dataDir: String): ByteArray?

    external fun init(dataDir: String)

    external fun bitwardenStatus(): String

    external fun bitwardenSignIn(server: String, email: String, password: String): String

    external fun bitwardenAnswer(method: String, code: String): String

    external fun bitwardenSendEmailCode(): String

    external fun bitwardenUnlock(password: String): String

    external fun bitwardenLock(): String

    external fun bitwardenMatches(url: String): String

    external fun bitwardenSearch(query: String): String

    external fun bitwardenTotp(secret: String): String

    external fun bitwardenGeneratePassword(): String

    external fun updateDue(dataDir: String): Boolean

    external fun updateCheck(dataDir: String, version: String, abis: String): String
}
