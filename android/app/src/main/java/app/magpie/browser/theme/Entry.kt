package app.magpie.browser.theme

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private val TOGGLE_WIDTH = 34.dp
private val TOGGLE_HEIGHT = 20.dp
private val TOGGLE_INSET = 3.dp
private val TOGGLE_KNOB = 14.dp
private val TOGGLE_TRAVEL = TOGGLE_WIDTH - TOGGLE_KNOB - TOGGLE_INSET * 2
private val TOGGLE_SHAPE = RoundedCornerShape(6.dp)
private val KNOB_SHAPE = RoundedCornerShape(4.dp)

@Composable
fun Section(text: String) {
    Text(
        text,
        modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 16.dp, bottom = 4.dp),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
fun Entry(
    icon: ImageVector?,
    label: String,
    onClick: () -> Unit,
    detail: String? = null,
    enabled: Boolean = true,
    trailing: @Composable (() -> Unit)? = null,
) {
    val color = if (enabled) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.outline
    Row(
        Modifier.fillMaxWidth().clickable(enabled = enabled, onClick = onClick).padding(horizontal = 20.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        icon?.let { Icon(it, contentDescription = null, tint = color, modifier = Modifier.size(22.dp)) }
        Column(Modifier.weight(1f)) {
            Text(label, color = color, maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.bodyLarge)
            detail?.let {
                Text(
                    it,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        trailing?.invoke()
    }
}

@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit) {
    val colors = MaterialTheme.colorScheme
    val knobOffset by animateDpAsState(if (checked) TOGGLE_TRAVEL else 0.dp, label = "toggle")
    Box(
        Modifier.size(TOGGLE_WIDTH, TOGGLE_HEIGHT)
            .clip(TOGGLE_SHAPE)
            .background(if (checked) colors.onSurface else colors.surfaceVariant)
            .toggleable(value = checked, role = Role.Switch, onValueChange = onChange)
            .padding(TOGGLE_INSET),
    ) {
        Box(
            Modifier.offset(x = knobOffset)
                .size(TOGGLE_KNOB)
                .clip(KNOB_SHAPE)
                .background(if (checked) colors.surface else colors.onSurfaceVariant),
        )
    }
}

@Composable
fun ToggleEntry(icon: ImageVector?, label: String, checked: Boolean, onChange: (Boolean) -> Unit, detail: String? = null) {
    Entry(icon, label, onClick = { onChange(!checked) }, detail = detail) { Toggle(checked, onChange) }
}

@Composable
fun RadioEntry(label: String, selected: Boolean, onSelect: () -> Unit, detail: String? = null) {
    Entry(null, label, onClick = onSelect, detail = detail) {
        RadioButton(selected = selected, onClick = onSelect)
    }
}
