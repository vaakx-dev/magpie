package app.magpie.browser.theme

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

enum class Outline { Solid, Dashed }

private val SIZE = 20.dp
private val STROKE = 2.dp
private val CORNER = 5.dp
private val DASH = floatArrayOf(6f, 4f)
private val LABEL = TextStyle(fontSize = 11.sp, fontWeight = FontWeight.Bold, lineHeight = 11.sp)

@Composable
fun Count(count: Int, outline: Outline = Outline.Solid) {
    val color = LocalContentColor.current
    Box(
        Modifier.size(SIZE).drawBehind {
            val width = STROKE.toPx()
            val dashes = if (outline == Outline.Dashed) PathEffect.dashPathEffect(DASH) else null
            drawRoundRect(
                color,
                topLeft = Offset(width / 2, width / 2),
                size = Size(size.width - width, size.height - width),
                cornerRadius = CornerRadius(CORNER.toPx()),
                style = Stroke(width, pathEffect = dashes),
            )
        },
        contentAlignment = Alignment.Center,
    ) {
        Text(count.toString(), style = LABEL, color = color)
    }
}
