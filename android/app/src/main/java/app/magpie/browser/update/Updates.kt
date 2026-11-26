package app.magpie.browser.update

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import app.magpie.browser.core.MagpieCore
import app.magpie.browser.download.contentUri
import app.magpie.browser.settings.Settings
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException

private const val APK_TYPE = "application/vnd.android.package-archive"
private const val UPDATES_FOLDER = "updates"
private const val NO_CORE = "Needs Magpie's core library, which isn't in this build."
private const val ALLOW_INSTALLS = "Allow Magpie to install apps, then tap Download & Install again"

class Updates(private val context: Context, private val scope: CoroutineScope, private val settings: Settings) {
    var phase by mutableStateOf<Phase>(Phase.Idle)
        private set
    private var dismissed by mutableStateOf(false)

    val version: String = context.packageManager.getPackageInfo(context.packageName, 0).versionName.orEmpty()
    private val dataDir = context.filesDir.absolutePath
    private val folder = File(context.cacheDir, UPDATES_FOLDER)

    private val busy: Boolean get() = phase == Phase.Checking || phase is Phase.Downloading

    val installable: Offer?
        get() = phase.offer?.takeIf { it.download != null && phase !is Phase.Downloading }

    val announced: Offer? get() = phase.offer?.takeIf { !dismissed && phase !is Phase.Installing }

    fun checkIfDue() {
        if (!settings.autoUpdate || !MagpieCore.loaded) return
        scope.launch {
            if (withContext(Dispatchers.IO) { MagpieCore.updateDue(dataDir) }) check()
        }
    }

    fun check() {
        if (busy) return
        if (!MagpieCore.loaded) {
            phase = Phase.Failed(NO_CORE)
            return
        }
        phase = Phase.Checking
        scope.launch {
            val abis = Build.SUPPORTED_ABIS.joinToString(",")
            phase = withContext(Dispatchers.IO) { parseStatus(MagpieCore.updateCheck(dataDir, version, abis)) }
            dismissed = false
        }
    }

    fun install() {
        val offer = installable ?: return
        val download = offer.download ?: return
        if (!context.packageManager.canRequestPackageInstalls()) {
            askToInstallApps()
            phase = Phase.DownloadFailed(offer, ALLOW_INSTALLS)
            return
        }
        phase = Phase.Downloading(offer, 0)
        scope.launch {
            phase = try {
                val apk = withContext(Dispatchers.IO) { fetch(download, folder) { percent -> scope.launch { progressed(percent) } } }
                openInstaller(apk)
                Phase.Installing(offer)
            } catch (failure: IOException) {
                Phase.DownloadFailed(offer, failure.message ?: "The download stopped")
            }
        }
    }

    fun dismiss() {
        dismissed = true
    }

    private fun progressed(percent: Int) {
        val downloading = phase
        if (downloading is Phase.Downloading) phase = downloading.copy(percent = percent)
    }

    private fun askToInstallApps() {
        val intent = Intent(ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${context.packageName}"))
        context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }

    private fun openInstaller(apk: File) {
        val intent = Intent(Intent.ACTION_VIEW)
            .setDataAndType(contentUri(context, apk), APK_TYPE)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
        context.startActivity(intent)
    }
}
