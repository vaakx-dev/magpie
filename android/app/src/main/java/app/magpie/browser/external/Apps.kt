package app.magpie.browser.external

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import java.net.URISyntaxException

class Apps(private val context: Context) {
    fun open(uri: String): Boolean {
        val intent = intent(uri) ?: return false
        return try {
            context.startActivity(intent)
            true
        } catch (missing: ActivityNotFoundException) {
            false
        } catch (forbidden: SecurityException) {
            false
        }
    }

    fun unavailable() = Toast.makeText(context, "No app can open this link", Toast.LENGTH_SHORT).show()

    private fun parsed(uri: String): Intent? {
        if (!appIntent(uri)) return Intent(Intent.ACTION_VIEW, Uri.parse(uri))
        return try {
            Intent.parseUri(uri, Intent.URI_INTENT_SCHEME)
        } catch (malformed: URISyntaxException) {
            null
        }
    }

    private fun intent(uri: String): Intent? {
        val intent = parsed(uri) ?: return null
        if (intent.`package` == context.packageName) return null
        return intent.apply {
            component = null
            selector = null
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
            addCategory(Intent.CATEGORY_BROWSABLE)
        }
    }
}
