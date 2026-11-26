package app.magpie.browser.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.KeyboardActionHandler
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldColors
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp

private val SHAPE = RoundedCornerShape(8.dp)
private val CHOICES_SHAPE = RoundedCornerShape(8.dp)
private val CHOICE_SHAPE = RoundedCornerShape(6.dp)
private val FIELD_PADDING = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 4.dp)

@Composable
private fun colors(): TextFieldColors = TextFieldDefaults.colors(
    focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
    focusedIndicatorColor = Color.Transparent,
    unfocusedIndicatorColor = Color.Transparent,
    focusedLabelColor = MaterialTheme.colorScheme.onSurfaceVariant,
    cursorColor = MaterialTheme.colorScheme.onSurface,
)

@Composable
fun Field(state: TextFieldState, label: String, keyboard: KeyboardType = KeyboardType.Text, onDone: (() -> Unit)? = null) {
    TextField(
        state = state,
        label = { Text(label) },
        lineLimits = TextFieldLineLimits.SingleLine,
        shape = SHAPE,
        keyboardOptions = KeyboardOptions(
            keyboardType = keyboard,
            imeAction = if (onDone == null) ImeAction.Default else ImeAction.Done,
            autoCorrectEnabled = false,
        ),
        onKeyboardAction = onDone?.let { done -> KeyboardActionHandler { done() } },
        colors = colors(),
        modifier = FIELD_PADDING,
    )
}

@Composable
fun SecretField(state: TextFieldState, label: String) {
    SecureTextField(state = state, label = { Text(label) }, shape = SHAPE, colors = colors(), modifier = FIELD_PADDING)
}

@Composable
fun <T> Choices(options: List<T>, selected: T, label: (T) -> String, onSelect: (T) -> Unit) {
    Row(
        Modifier.fillMaxWidth()
            .padding(horizontal = 20.dp, vertical = 6.dp)
            .clip(CHOICES_SHAPE)
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(3.dp),
        horizontalArrangement = Arrangement.spacedBy(3.dp),
    ) {
        options.forEach { option ->
            val chosen = option == selected
            Box(
                Modifier.weight(1f)
                    .clip(CHOICE_SHAPE)
                    .background(if (chosen) MaterialTheme.colorScheme.onSurface else Color.Transparent)
                    .clickable { onSelect(option) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    label(option),
                    style = MaterialTheme.typography.labelLarge,
                    color = if (chosen) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
