package app.magpie.browser.address

import android.content.Context
import android.content.Intent
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldDecorator
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Edit
import androidx.compose.material.icons.outlined.History
import androidx.compose.material.icons.outlined.Language
import androidx.compose.material.icons.outlined.NorthWest
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.magpie.browser.browser.Browser
import app.magpie.browser.browser.Tab
import app.magpie.browser.clipboard.Content
import app.magpie.browser.clipboard.copy
import app.magpie.browser.favorite.Grid
import app.magpie.browser.theme.Letter

private val FIELD_SHAPE = RoundedCornerShape(8.dp)
private val CARD_SHAPE = RoundedCornerShape(12.dp)
private val FIELD_HEIGHT = 46.dp
private val ROW_HEIGHT = 52.dp

private val SOURCE_ICONS = mapOf(
    Source.Site to Icons.Outlined.Language,
    Source.Search to Icons.Outlined.Search,
    Source.Bookmark to Icons.Outlined.StarBorder,
    Source.History to Icons.Outlined.History,
)

private fun share(context: Context, tab: Tab) {
    val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, tab.url).putExtra(Intent.EXTRA_SUBJECT, tab.title)
    context.startActivity(Intent.createChooser(send, null))
}

private fun copyLink(context: Context, tab: Tab) {
    copy(context, "Link", tab.url, Content.Plain)
    Toast.makeText(context, "Link copied", Toast.LENGTH_SHORT).show()
}

private fun emphasised(label: String, query: String, strong: SpanStyle): AnnotatedString = buildAnnotatedString {
    val text = query.trim()
    val start = if (text.isEmpty()) -1 else label.indexOf(text, ignoreCase = true)
    if (start < 0) {
        withStyle(strong) { append(label) }
        return@buildAnnotatedString
    }
    val end = start + text.length
    withStyle(strong) { append(label.substring(0, start)) }
    append(label.substring(start, end))
    withStyle(strong) { append(label.substring(end)) }
}

@Composable
private fun Action(icon: ImageVector, description: String, onClick: () -> Unit) {
    IconButton(onClick = onClick, shape = FIELD_SHAPE, modifier = Modifier.size(40.dp)) {
        Icon(icon, description, Modifier.size(19.dp), tint = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
private fun Omnibox(state: TextFieldState, focus: FocusRequester, onBack: () -> Unit, onGo: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.padding(start = 10.dp, end = 10.dp, top = 10.dp).fillMaxWidth().height(FIELD_HEIGHT).clip(FIELD_SHAPE).background(colors.surface),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Action(Icons.AutoMirrored.Outlined.ArrowBack, "Back", onBack)
        BasicTextField(
            state = state,
            lineLimits = TextFieldLineLimits.SingleLine,
            textStyle = MaterialTheme.typography.bodyLarge.copy(color = colors.onSurface),
            cursorBrush = SolidColor(colors.onSurface),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Go, autoCorrectEnabled = false),
            onKeyboardAction = { onGo() },
            decorator = TextFieldDecorator { field ->
                Box(contentAlignment = Alignment.CenterStart) {
                    if (state.text.isEmpty()) Text(PLACEHOLDER, style = MaterialTheme.typography.bodyLarge, color = colors.onSurfaceVariant)
                    field()
                }
            },
            modifier = Modifier.weight(1f).padding(horizontal = 4.dp).focusRequester(focus),
        )
        if (state.text.isNotEmpty()) Action(Icons.Outlined.Close, "Clear") { state.clearText() }
    }
}

@Composable
private fun Current(tab: Tab, onEdit: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val context = LocalContext.current
    Row(
        Modifier.padding(start = 10.dp, end = 10.dp, top = 8.dp).fillMaxWidth().clip(CARD_SHAPE).background(colors.surface)
            .padding(start = 12.dp, end = 4.dp, top = 10.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Letter(tab.url, 34.dp)
        Column(Modifier.weight(1f)) {
            Text(tab.label, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 14.sp, fontWeight = FontWeight.Medium)
            Text(tab.url, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.5.sp, color = colors.onSurfaceVariant)
        }
        Row {
            Action(Icons.Outlined.Share, "Share") { share(context, tab) }
            Action(Icons.Outlined.ContentCopy, "Copy URL") { copyLink(context, tab) }
            Action(Icons.Outlined.Edit, "Edit URL", onEdit)
        }
    }
}

@Composable
private fun Label(text: String) {
    Text(
        text,
        Modifier.padding(start = 18.dp, end = 18.dp, top = 12.dp, bottom = 4.dp),
        fontSize = 11.5.sp,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun Line(suggestion: Suggestion, query: String, onOpen: () -> Unit, onFill: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.fillMaxWidth().height(ROW_HEIGHT).clickable(onClick = onOpen).padding(start = 18.dp, end = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Icon(SOURCE_ICONS.getValue(suggestion.source), contentDescription = null, Modifier.size(19.dp), tint = colors.onSurfaceVariant)
        Column(Modifier.weight(1f)) {
            Text(
                emphasised(suggestion.label, query, SpanStyle(color = colors.onSurface, fontWeight = FontWeight.Medium)),
                fontSize = 14.sp,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(suggestion.detail, fontSize = 12.sp, color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        IconButton(onClick = onFill, shape = FIELD_SHAPE, modifier = Modifier.size(36.dp)) {
            Icon(Icons.Outlined.NorthWest, "Edit", Modifier.size(18.dp), tint = colors.onSurfaceVariant)
        }
    }
}

@Composable
fun Typing(browser: Browser, onDismiss: () -> Unit) {
    val state = rememberTextFieldState()
    val focus = remember { FocusRequester() }
    val page = browser.tab?.takeUnless { it.blank }
    val favorites = browser.favorites.list
    val query = state.text.toString()
    val open = { target: String ->
        browser.load(target)
        onDismiss()
    }
    LaunchedEffect(Unit) { focus.requestFocus() }
    BackHandler(onBack = onDismiss)
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).systemBarsPadding().imePadding()) {
        Omnibox(state, focus, onBack = onDismiss, onGo = { open(query) })
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 8.dp)) {
            page?.let { tab -> item { Current(tab, onEdit = { state.setTextAndPlaceCursorAtEnd(tab.url) }) } }
            if (favorites.isNotEmpty()) item { Grid(browser.favorites) { open(it.url) } }
            val matches = suggestions(query, favorites, browser.history.visits)
            if (matches.isNotEmpty()) item { Label(if (query.isBlank()) "Recent" else "Suggestions") }
            items(matches, key = { "${it.source}:${it.target}" }) { suggestion ->
                Line(suggestion, query, onOpen = { open(suggestion.target) }, onFill = { state.setTextAndPlaceCursorAtEnd(suggestion.target) })
            }
        }
    }
}
