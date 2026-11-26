package app.magpie.browser.password

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricPrompt
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.util.readBytes
import androidx.core.util.writeBytes
import androidx.fragment.app.FragmentActivity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import java.security.GeneralSecurityException
import java.security.Key
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.GCMParameterSpec
import kotlin.coroutines.resume

sealed interface Approval {
    val problem: String

    data object Granted : Approval {
        override val problem = ""
    }

    data object Cancelled : Approval {
        override val problem = ""
    }

    data class Refused(override val problem: String) : Approval
}

sealed interface Opening {
    class Opened(val secret: ByteArray) : Opening

    data class Closed(val approval: Approval) : Opening

    data object Expired : Opening
}

private const val KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "magpie.vault.biometric"
private const val KEY_FILE = "vault-biometric.key"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val KEY_BITS = 256
private const val TAG_BITS = 128
private const val IV_BYTES = 12
private const val PROMPT_SUBTITLE = "Bitwarden vault"
private const val PROMPT_CANCEL = "Use master password"
private val CANCELLED = setOf(
    BiometricPrompt.ERROR_USER_CANCELED,
    BiometricPrompt.ERROR_NEGATIVE_BUTTON,
    BiometricPrompt.ERROR_CANCELED,
)

private fun keyStore(): KeyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }

private fun freshKey(): Key {
    val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(KEY_BITS)
        .setUserAuthenticationRequired(true)
        .setInvalidatedByBiometricEnrollment(true)
        .build()
    return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply { init(spec) }.generateKey()
}

private fun promptInfo(title: String): BiometricPrompt.PromptInfo = BiometricPrompt.PromptInfo.Builder()
    .setTitle(title)
    .setSubtitle(PROMPT_SUBTITLE)
    .setNegativeButtonText(PROMPT_CANCEL)
    .setAllowedAuthenticators(BIOMETRIC_STRONG)
    .build()

private suspend fun approve(activity: FragmentActivity, cipher: Cipher, title: String): Approval =
    suspendCancellableCoroutine { continuation ->
        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                continuation.resume(Approval.Granted)
            }

            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                continuation.resume(if (errorCode in CANCELLED) Approval.Cancelled else Approval.Refused(errString.toString()))
            }
        }
        val prompt = BiometricPrompt(activity, callback)
        prompt.authenticate(promptInfo(title), BiometricPrompt.CryptoObject(cipher))
        continuation.invokeOnCancellation { prompt.cancelAuthentication() }
    }

private fun decrypting(key: Key, iv: ByteArray): Cipher? = try {
    Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(TAG_BITS, iv)) }
} catch (invalidated: KeyPermanentlyInvalidatedException) {
    null
}

class Biometrics(context: Context) {
    private val manager = BiometricManager.from(context)
    private val file = AtomicFile(File(context.filesDir, KEY_FILE))

    var on by mutableStateOf(file.baseFile.exists())
        private set

    val available: Boolean get() = manager.canAuthenticate(BIOMETRIC_STRONG) == BiometricManager.BIOMETRIC_SUCCESS

    suspend fun seal(activity: FragmentActivity, secret: ByteArray): Approval {
        val cipher = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, freshKey()) }
        val approval = approve(activity, cipher, "Turn on biometric unlock")
        if (approval != Approval.Granted) return approval
        val sealed = cipher.iv + cipher.doFinal(secret)
        withContext(Dispatchers.IO) { file.writeBytes(sealed) }
        on = true
        return approval
    }

    suspend fun open(activity: FragmentActivity): Opening {
        val sealed = withContext(Dispatchers.IO) { sealed() } ?: return expire()
        val key = keyStore().getKey(KEY_ALIAS, null) ?: return expire()
        val cipher = decrypting(key, sealed.copyOfRange(0, IV_BYTES)) ?: return expire()
        val approval = approve(activity, cipher, "Unlock your vault")
        if (approval != Approval.Granted) return Opening.Closed(approval)
        return try {
            Opening.Opened(cipher.doFinal(sealed, IV_BYTES, sealed.size - IV_BYTES))
        } catch (tampered: GeneralSecurityException) {
            expire()
        }
    }

    fun forget() {
        file.delete()
        keyStore().deleteEntry(KEY_ALIAS)
        on = false
    }

    private fun sealed(): ByteArray? = try {
        file.readBytes().takeIf { it.size > IV_BYTES }
    } catch (missing: IOException) {
        null
    }

    private fun expire(): Opening {
        forget()
        return Opening.Expired
    }
}
