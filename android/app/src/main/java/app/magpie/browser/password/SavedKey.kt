package app.magpie.browser.password

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import androidx.core.util.readBytes
import androidx.core.util.writeBytes
import java.io.File
import java.io.IOException
import java.security.GeneralSecurityException
import java.security.Key
import java.security.KeyStore
import java.security.UnrecoverableKeyException
import javax.crypto.AEADBadTagException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.GCMParameterSpec

private const val KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "magpie.vault.key"
private const val KEY_FILE = "vault.key"
private const val LEGACY_KEY_ALIAS = "magpie.vault.biometric"
private const val LEGACY_KEY_FILE = "vault-biometric.key"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val KEY_BITS = 256
private const val TAG_BITS = 128
private const val IV_BYTES = 12

private sealed interface Opened {
    class Key(val secret: ByteArray) : Opened

    data object Gone : Opened

    data object Unavailable : Opened
}

private fun keyStore(): KeyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }

private fun freshKey(): Key {
    val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(KEY_BITS)
        .build()
    return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply { init(spec) }.generateKey()
}

private fun decrypted(sealed: ByteArray): Opened = try {
    val key = keyStore().getKey(KEY_ALIAS, null) ?: return Opened.Gone
    val cipher = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(TAG_BITS, sealed, 0, IV_BYTES)) }
    Opened.Key(cipher.doFinal(sealed, IV_BYTES, sealed.size - IV_BYTES))
} catch (invalidated: KeyPermanentlyInvalidatedException) {
    Opened.Gone
} catch (unrecoverable: UnrecoverableKeyException) {
    Opened.Gone
} catch (mismatched: AEADBadTagException) {
    Opened.Gone
} catch (busy: GeneralSecurityException) {
    Opened.Unavailable
} catch (busy: IOException) {
    Opened.Unavailable
} catch (busy: RuntimeException) {
    Opened.Unavailable
}

private fun deleteEntry(alias: String) {
    runCatching { keyStore().deleteEntry(alias) }
}

class SavedKey(context: Context) {
    private val file = AtomicFile(File(context.filesDir, KEY_FILE))
    private val legacy = File(context.filesDir, LEGACY_KEY_FILE)

    fun save(secret: ByteArray): Boolean = try {
        val cipher = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, freshKey()) }
        file.writeBytes(cipher.iv + cipher.doFinal(secret))
        true
    } catch (unsealable: GeneralSecurityException) {
        false
    } catch (unwritable: IOException) {
        false
    } catch (busy: RuntimeException) {
        false
    }

    fun open(): ByteArray? {
        val sealed = sealed() ?: return null
        return when (val opened = decrypted(sealed)) {
            is Opened.Key -> opened.secret
            Opened.Gone -> null.also { forget() }
            Opened.Unavailable -> null
        }
    }

    fun forget() {
        file.delete()
        deleteEntry(KEY_ALIAS)
    }

    fun dropLegacy() {
        if (!legacy.exists()) return
        AtomicFile(legacy).delete()
        deleteEntry(LEGACY_KEY_ALIAS)
    }

    private fun sealed(): ByteArray? = try {
        file.readBytes().takeIf { it.size > IV_BYTES }
    } catch (missing: IOException) {
        null
    }
}
