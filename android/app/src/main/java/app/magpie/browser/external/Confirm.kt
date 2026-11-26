package app.magpie.browser.external

import android.app.Activity
import android.app.AlertDialog

fun confirmAppLink(activity: Activity, open: () -> Unit, dismiss: () -> Unit): AlertDialog =
    AlertDialog.Builder(activity)
        .setTitle("Open another app?")
        .setMessage("This page wants to open a link outside Magpie.")
        .setPositiveButton("Open") { _, _ -> open() }
        .setNegativeButton("Cancel") { _, _ -> dismiss() }
        .setOnCancelListener { dismiss() }
        .show()
