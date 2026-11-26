package app.magpie.browser.find

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Tab
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSession.FinderResult

private val PILL = RoundedCornerShape(50)

@Composable
private fun Action(icon: ImageVector, description: String, onClick: () -> Unit) {
    IconButton(onClick = onClick) {
        Icon(icon, contentDescription = description, tint = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
fun FindBar(tab: Tab, onClose: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val finder = tab.session.finder
    val state = rememberTextFieldState()
    val focus = remember { FocusRequester() }
    var result by remember { mutableStateOf<FinderResult?>(null) }
    val search = { query: String?, flags: Int -> finder.find(query, flags).accept { result = it } }
    LaunchedEffect(finder) {
        focus.requestFocus()
        snapshotFlow { state.text.toString() }.collect { query ->
            if (query.isEmpty()) {
                finder.clear()
                result = null
                return@collect
            }
            search(query, 0)
        }
    }
    DisposableEffect(finder) { onDispose { finder.clear() } }
    BackHandler(onBack = onClose)
    Surface(
        shape = PILL,
        color = colors.surface,
        contentColor = colors.onSurface,
        shadowElevation = 3.dp,
        modifier = Modifier.fillMaxWidth().padding(horizontal = 10.dp, vertical = 8.dp).height(46.dp),
    ) {
        Row(Modifier.padding(start = 18.dp, end = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            BasicTextField(
                state = state,
                lineLimits = TextFieldLineLimits.SingleLine,
                textStyle = MaterialTheme.typography.bodyLarge.copy(color = colors.onSurface),
                cursorBrush = SolidColor(colors.onSurface),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search, autoCorrectEnabled = false),
                onKeyboardAction = { search(null, 0) },
                decorator = { field ->
                    Box(contentAlignment = Alignment.CenterStart) {
                        if (state.text.isEmpty()) Text("Find in page", color = colors.onSurfaceVariant, style = MaterialTheme.typography.bodyLarge)
                        field()
                    }
                },
                modifier = Modifier.weight(1f).focusRequester(focus),
            )
            result?.let { Text("${it.current}/${it.total}", color = colors.onSurfaceVariant, style = MaterialTheme.typography.bodySmall) }
            Action(Icons.Outlined.KeyboardArrowUp, "Previous match") { search(null, GeckoSession.FINDER_FIND_BACKWARDS) }
            Action(Icons.Outlined.KeyboardArrowDown, "Next match") { search(null, 0) }
            Action(Icons.Outlined.Close, "Close find", onClose)
        }
    }
}
