package app.magpie.browser.external

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LinkTest {
    @Test
    fun handsAppLinksToAndroid() {
        listOf(
            "mailto:someone@example.com",
            "tel:+61400000000",
            "sms:+61400000000?body=hi",
            "geo:-33.86,151.20",
            "intent://scan/#Intent;scheme=zxing;package=com.example.scanner;end",
            "MAILTO:someone@example.com",
            "market://details?id=com.example",
        ).forEach { assertTrue(it, leavesBrowser(it)) }
    }

    @Test
    fun keepsWebAndEngineLinksInTheBrowser() {
        listOf(
            "https://example.com/mailto:x",
            "http://example.com",
            "about:blank",
            "javascript:alert(1)",
            "data:text/html,hi",
            "blob:https://example.com/1234",
            "file:///sdcard/page.html",
            "content://app.magpie.browser.files/secret",
            "not a link",
        ).forEach { assertFalse(it, leavesBrowser(it)) }
    }

    @Test
    fun readsWebFallbackFromAppIntents() {
        val intent = "intent://open/#Intent;scheme=example;package=com.example;" +
            "S.browser_fallback_url=https%3A%2F%2Fexample.com%2Fget%3Fapp%3D1;end"
        assertEquals("https://example.com/get?app=1", fallback(intent))
    }

    @Test
    fun ignoresUnsafeOrMissingFallbacks() {
        assertNull(fallback("intent://open/#Intent;scheme=example;S.browser_fallback_url=javascript%3Aalert(1);end"))
        assertNull(fallback("intent://open/#Intent;scheme=example;S.browser_fallback_url=file%3A%2F%2F%2Fsdcard;end"))
        assertNull(fallback("intent://open/#Intent;scheme=example;package=com.example;end"))
        assertNull(fallback("https://example.com/#Intent;S.browser_fallback_url=https%3A%2F%2Fevil.example;end"))
    }
}
