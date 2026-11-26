package app.magpie.browser.onboarding

import android.app.role.RoleManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.provider.Settings
import android.view.autofill.AutofillManager

fun autofillOn(context: Context): Boolean = context.getSystemService(AutofillManager::class.java).isEnabled

fun defaultBrowserRequest(context: Context): Intent =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        context.getSystemService(RoleManager::class.java).createRequestRoleIntent(RoleManager.ROLE_BROWSER)
    } else {
        Intent(Settings.ACTION_MANAGE_DEFAULT_APPS_SETTINGS)
    }
