package app.magpie.browser.favorite

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Edit
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.unit.dp

private val SHAPE = RoundedCornerShape(12.dp)

@Composable
fun FavoriteMenu(favorite: Favorite, favorites: Favorites, expanded: Boolean, onDismiss: () -> Unit) {
    var editing by remember { mutableStateOf(false) }
    DropdownMenu(expanded, onDismiss, shape = SHAPE, containerColor = MaterialTheme.colorScheme.surface, tonalElevation = 0.dp) {
        DropdownMenuItem(
            text = { Text("Edit bookmark") },
            leadingIcon = { Icon(Icons.Outlined.Edit, contentDescription = null) },
            onClick = {
                onDismiss()
                editing = true
            },
        )
        DropdownMenuItem(
            text = { Text("Remove bookmark") },
            leadingIcon = { Icon(Icons.Outlined.Close, contentDescription = null) },
            onClick = {
                onDismiss()
                favorites.remove(favorite)
            },
        )
    }
    if (editing) {
        EditDialog(
            favorite,
            onSave = {
                favorites.replace(favorite, it)
                editing = false
            },
            onDismiss = { editing = false },
        )
    }
}
