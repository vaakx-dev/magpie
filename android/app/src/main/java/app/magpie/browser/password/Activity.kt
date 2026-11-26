package app.magpie.browser.password

import androidx.activity.compose.LocalActivity
import androidx.compose.runtime.Composable
import androidx.fragment.app.FragmentActivity

@Composable
fun hostActivity(): FragmentActivity = LocalActivity.current as FragmentActivity
