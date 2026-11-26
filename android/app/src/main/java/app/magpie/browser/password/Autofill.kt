package app.magpie.browser.password

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import android.widget.Toast

private const val NO_PICKER = "Change it in Android Settings under Passwords & autofill."

fun changeAutofillService(context: Context) {
    val picker = Intent(Settings.ACTION_REQUEST_SET_AUTOFILL_SERVICE, Uri.fromParts("package", context.packageName, null))
    try {
        context.startActivity(picker)
    } catch (_: ActivityNotFoundException) {
        Toast.makeText(context, NO_PICKER, Toast.LENGTH_LONG).show()
    }
}
