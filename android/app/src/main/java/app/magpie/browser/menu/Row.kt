package app.magpie.browser.menu

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.magpie.browser.favicon.Favicon
import app.magpie.browser.theme.CONTROL_SHAPE
import app.magpie.browser.theme.IconAction

val PANEL_PADDING = 4.dp
val ROW_HEIGHT = 44.dp
val ICON_SIZE = 20.dp

private val DIVIDER_INSET = 6.dp
private val DIVIDER_SIDE = DIVIDER_INSET - PANEL_PADDING
private val LIST_MAX_HEIGHT = 336.dp

@Composable
fun Divider() {
    HorizontalDivider(
        Modifier.padding(horizontal = DIVIDER_SIDE, vertical = 2.dp),
        color = MaterialTheme.colorScheme.outlineVariant,
    )
}

@Composable
fun Heading(text: String) {
    Text(
        text,
        modifier = Modifier.padding(start = 10.dp, end = 10.dp, top = 6.dp, bottom = 2.dp),
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
fun Line(
    leading: @Composable () -> Unit,
    label: String,
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurface,
    trailing: @Composable RowScope.() -> Unit = {},
) {
    Row(
        Modifier.fillMaxWidth().height(ROW_HEIGHT).clip(CONTROL_SHAPE).then(modifier).padding(horizontal = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        leading()
        Text(
            label,
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyLarge,
            modifier = Modifier.weight(1f),
        )
        trailing()
    }
}

@Composable
fun Glyph(icon: ImageVector, tint: Color = MaterialTheme.colorScheme.onSurfaceVariant) {
    Icon(icon, contentDescription = null, tint = tint, modifier = Modifier.size(ICON_SIZE))
}

@Composable
fun Item(icon: ImageVector, label: String, onClick: (() -> Unit)?, trailing: @Composable RowScope.() -> Unit = {}) {
    val colors = MaterialTheme.colorScheme
    val enabled = onClick != null
    Line(
        leading = { Glyph(icon, if (enabled) colors.onSurfaceVariant else colors.outline) },
        label = label,
        modifier = Modifier.clickable(enabled = enabled, onClick = onClick ?: {}),
        color = if (enabled) colors.onSurface else colors.outline,
        trailing = trailing,
    )
}

@Composable
fun Drill(icon: ImageVector, label: String, onEnter: () -> Unit) {
    Item(icon, label, onClick = onEnter) { Glyph(Icons.AutoMirrored.Outlined.KeyboardArrowRight) }
}

@Composable
fun Link(url: String, title: String, onOpen: () -> Unit) {
    Line(leading = { Favicon(url, ICON_SIZE) }, label = title.ifBlank { url }, modifier = Modifier.clickable(onClick = onOpen))
}

@Composable
fun Header(title: String, onBack: () -> Unit) {
    val color = MaterialTheme.colorScheme.onSurface
    Row(Modifier.fillMaxWidth().height(ROW_HEIGHT), verticalAlignment = Alignment.CenterVertically) {
        IconAction(Icons.AutoMirrored.Outlined.KeyboardArrowLeft, "Back", onBack, tint = color)
        Text(
            title,
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.weight(1f),
        )
        Box(Modifier.width(ROW_HEIGHT))
    }
    Divider()
}

@Composable
fun ScrollList(content: @Composable ColumnScope.() -> Unit) {
    Column(Modifier.heightIn(max = LIST_MAX_HEIGHT).verticalScroll(rememberScrollState()), content = content)
}

@Composable
fun Detail(text: String) {
    Text(text, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
}
