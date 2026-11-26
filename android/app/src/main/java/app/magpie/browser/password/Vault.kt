package app.magpie.browser.password

import android.content.Context
import androidx.fragment.app.FragmentActivity
import app.magpie.browser.core.MagpieCore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

enum class Lock { Unavailable, SignedOut, Challenged, Locked, Unlocked }

enum class Method(val key: String, val label: String) {
    Authenticator("authenticator", "Authenticator"),
    Email("email", "Email"),
    YubiKey("yubiKey", "YubiKey"),
    NewDevice("newDevice", "New device"),
}

data class Status(
    val lock: Lock,
    val email: String = "",
    val methods: List<Method> = emptyList(),
    val error: String = "",
) {
    val signedIn: Boolean get() = lock == Lock.Locked || lock == Lock.Unlocked
}

data class Login(val id: String, val name: String, val username: String, val password: String, val totp: String)

enum class Server(val label: String, val address: String) {
    UnitedStates("bitwarden.com", "https://vault.bitwarden.com"),
    Europe("bitwarden.eu", "https://vault.bitwarden.eu"),
    SelfHosted("Self-hosted", ""),
}

private val LOCKS = mapOf(
    "signedOut" to Lock.SignedOut,
    "challenged" to Lock.Challenged,
    "locked" to Lock.Locked,
    "unlocked" to Lock.Unlocked,
)

private val METHODS = Method.entries.associateBy(Method::key)

private fun methods(array: JSONArray): List<Method> = (0 until array.length()).mapNotNull { METHODS[array.getString(it)] }

fun parseStatus(json: String): Status {
    val reply = JSONObject(json)
    return Status(
        lock = LOCKS[reply.optString("state")] ?: Lock.SignedOut,
        email = reply.optString("email"),
        methods = methods(reply.optJSONArray("methods") ?: JSONArray()),
        error = reply.optString("error"),
    )
}

fun parseLogins(json: String): List<Login> {
    val array = JSONArray(json)
    return (0 until array.length()).map { array.getJSONObject(it) }.map {
        Login(
            id = it.getString("id"),
            name = it.optString("name"),
            username = it.optString("username"),
            password = it.optString("password"),
            totp = it.optString("totp"),
        )
    }
}

fun parseCode(json: String): String = JSONObject(json).optString("code")

private const val LOCKED_FIRST = "Unlock the vault with your master password first."
private const val BIOMETRICS_EXPIRED = "Biometric unlock stopped working and was turned off. Unlock with your master password, then turn it back on."

class Vault(context: Context) {
    val biometrics = Biometrics(context)

    suspend fun status(): Status =
        if (MagpieCore.loaded) core { parseStatus(MagpieCore.bitwardenStatus()) } else Status(Lock.Unavailable)

    suspend fun signIn(server: String, email: String, password: String): Status =
        core { parseStatus(MagpieCore.bitwardenSignIn(server, email, password)) }

    suspend fun answer(method: Method, code: String): Status = core { parseStatus(MagpieCore.bitwardenAnswer(method.key, code)) }

    suspend fun sendEmailCode(): Status = core { parseStatus(MagpieCore.bitwardenSendEmailCode()) }

    suspend fun unlock(password: String): Status = core { parseStatus(MagpieCore.bitwardenUnlock(password)) }

    suspend fun lock(): Status = core { parseStatus(MagpieCore.bitwardenLock()) }

    suspend fun signOut(): Status {
        biometrics.forget()
        return core { parseStatus(MagpieCore.bitwardenSignOut()) }
    }

    suspend fun turnOnBiometrics(activity: FragmentActivity): String {
        val key = core { MagpieCore.bitwardenKey() } ?: return LOCKED_FIRST
        return try {
            biometrics.seal(activity, key).problem
        } finally {
            key.fill(0)
        }
    }

    suspend fun unlockWithBiometrics(activity: FragmentActivity, current: Status): Status =
        when (val opening = biometrics.open(activity)) {
            is Opening.Opened -> unlock(opening.secret)
            is Opening.Closed -> current.copy(error = opening.approval.problem)
            Opening.Expired -> current.copy(error = BIOMETRICS_EXPIRED)
        }

    private suspend fun unlock(key: ByteArray): Status {
        val status = try {
            core { parseStatus(MagpieCore.bitwardenUnlockWithKey(key)) }
        } finally {
            key.fill(0)
        }
        if (status.lock != Lock.Unlocked) biometrics.forget()
        return status
    }

    suspend fun matches(url: String): List<Login> = core { parseLogins(MagpieCore.bitwardenMatches(url)) }

    suspend fun search(query: String): List<Login> = core { parseLogins(MagpieCore.bitwardenSearch(query)) }

    suspend fun totp(secret: String): String = core { parseCode(MagpieCore.bitwardenTotp(secret)) }

    suspend fun generatePassword(): String = core { MagpieCore.bitwardenGeneratePassword() }

    private suspend fun <T> core(call: () -> T): T = withContext(Dispatchers.IO) { call() }
}
