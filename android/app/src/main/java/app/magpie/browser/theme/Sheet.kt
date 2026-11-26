package app.magpie.browser.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.SheetValue
import androidx.compose.material3.rememberBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private val SHAPE = RoundedCornerShape(topStart = CARD_CORNER, topEnd = CARD_CORNER)
private val HANDLE_SHAPE = RoundedCornerShape(2.dp)
@OptIn(ExperimentalMaterial3Api::class)
private val STOPS = setOf(SheetValue.Hidden, SheetValue.Expanded)

@Composable
private fun Handle() {
    Box(Modifier.padding(vertical = 10.dp).size(32.dp, 4.dp).clip(HANDLE_SHAPE).background(MaterialTheme.colorScheme.outlineVariant))
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Sheet(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberBottomSheetState(initialValue = SheetValue.Hidden, enabledValues = STOPS),
        shape = SHAPE,
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
        scrimColor = MaterialTheme.colorScheme.scrim,
        dragHandle = { Handle() },
    ) {
        Column(Modifier.navigationBarsPadding().padding(bottom = 12.dp), content = content)
    }
}

@Composable
fun Title(text: String, modifier: Modifier = Modifier) {
    Text(
        text,
        modifier = modifier.padding(horizontal = GUTTER, vertical = 8.dp),
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.SemiBold,
    )
}

@Composable
fun Note(text: String) {
    Text(
        text,
        modifier = Modifier.padding(horizontal = GUTTER, vertical = 12.dp),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}
