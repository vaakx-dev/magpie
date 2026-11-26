package app.magpie.browser.password

import org.junit.Assert.assertEquals
import org.junit.Test

class VaultReplyTest {
    @Test
    fun readsStatusReplies() {
        assertEquals(Status(Lock.Unlocked, email = "sam@example.com"), parseStatus("""{"state":"unlocked","email":"sam@example.com"}"""))
        assertEquals(
            Status(Lock.SignedOut, error = "Wrong master password"),
            parseStatus("""{"state":"signedOut","error":"Wrong master password"}"""),
        )
    }

    @Test
    fun readsTwoStepChallenges() {
        assertEquals(
            Status(Lock.Challenged, methods = listOf(Method.Authenticator, Method.Email), error = "That code didn't work."),
            parseStatus("""{"state":"challenged","methods":["authenticator","webAuthn","email"],"error":"That code didn't work."}"""),
        )
    }

    @Test
    fun readsLoginsWithMissingFields() {
        val logins = parseLogins("""[{"id":"1","name":"Example","username":"sam","password":"hunter2","totp":"JBSWY3DP"},{"id":"2","name":"Bare"}]""")
        assertEquals(
            listOf(
                Login("1", "Example", "sam", "hunter2", "JBSWY3DP"),
                Login("2", "Bare", "", "", ""),
            ),
            logins,
        )
    }
}
