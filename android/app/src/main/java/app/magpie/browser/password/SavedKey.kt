package app.magpie.browser.password

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import androidx.core.util.readBytes
import androidx.core.util.writeBytes
import java.io.File
import java.io.IOException
import java.security.GeneralSecurityException
import java.security.Key
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.GCMParameterSpec

private const val KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "magpie.vault.key"
private const val KEY_FILE = "vault.key"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val KEY_BITS = 256
private const val TAG_BITS = 128
private const val IV_BYTES = 12

private fun keyStore(): KeyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }

private fun freshKey(): Key {
    val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(KEY_BITS)
        .build()
    return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply { init(spec) }.generateKey()
}

private fun decrypted(sealed: ByteArray): ByteArray? = try {
    val key = keyStore().getKey(KEY_ALIAS, null) ?: return null
    val cipher = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(TAG_BITS, sealed, 0, IV_BYTES)) }
    cipher.doFinal(sealed, IV_BYTES, sealed.size - IV_BYTES)
} catch (unreadable: GeneralSecurityException) {
    null
}

class SavedKey(context: Context) {
    private val file = AtomicFile(File(context.filesDir, KEY_FILE))

    fun save(secret: ByteArray) {
        val cipher = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, freshKey()) }
        file.writeBytes(cipher.iv + cipher.doFinal(secret))
    }

    fun open(): ByteArray? {
        val sealed = sealed() ?: return null
        val secret = decrypted(sealed)
        if (secret == null) forget()
        return secret
    }

    fun forget() {
        file.delete()
        keyStore().deleteEntry(KEY_ALIAS)
    }

    private fun sealed(): ByteArray? = try {
        file.readBytes().takeIf { it.size > IV_BYTES }
    } catch (missing: IOException) {
        null
    }
}
