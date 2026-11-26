package app.magpie.browser.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.graphics.Color
import app.magpie.browser.settings.ThemeMode

private data class Palette(
    val frame: Color,
    val surface: Color,
    val raised: Color,
    val text: Color,
    val muted: Color,
    val faint: Color,
    val line: Color,
    val scrim: Color,
)

private val LIGHT = Palette(
    frame = Color(0xFFF3F3F3),
    surface = Color(0xFFFFFFFF),
    raised = Color(0xFFEAEAEA),
    text = Color(0xFF141414),
    muted = Color(0xFF666666),
    faint = Color(0xFFA3A3A3),
    line = Color(0xFFDCDCDC),
    scrim = Color(0x2E141414),
)

private val DARK = Palette(
    frame = Color(0xFF141414),
    surface = Color(0xFF1F1F1F),
    raised = Color(0xFF2A2A2A),
    text = Color(0xFFF2F2F2),
    muted = Color(0xFF969696),
    faint = Color(0xFF6B6B6B),
    line = Color(0xFF2E2E2E),
    scrim = Color(0x66000000),
)

private fun Palette.scheme(dark: Boolean): ColorScheme {
    val base = if (dark) darkColorScheme() else lightColorScheme()
    return base.copy(
        primary = text,
        onPrimary = surface,
        primaryContainer = raised,
        onPrimaryContainer = text,
        secondary = muted,
        onSecondary = surface,
        secondaryContainer = raised,
        onSecondaryContainer = text,
        tertiary = muted,
        onTertiary = surface,
        background = frame,
        onBackground = text,
        surface = surface,
        onSurface = text,
        surfaceVariant = raised,
        onSurfaceVariant = muted,
        surfaceTint = Color.Transparent,
        surfaceBright = surface,
        surfaceDim = frame,
        surfaceContainerLowest = surface,
        surfaceContainerLow = surface,
        surfaceContainer = surface,
        surfaceContainerHigh = surface,
        surfaceContainerHighest = raised,
        inverseSurface = text,
        inverseOnSurface = surface,
        outline = faint,
        outlineVariant = line,
        scrim = scrim,
        error = text,
        onError = surface,
    )
}

@Composable
fun ThemeMode.dark(): Boolean = when (this) {
    ThemeMode.System -> isSystemInDarkTheme()
    ThemeMode.Light -> false
    ThemeMode.Dark -> true
}

@Composable
fun MagpieTheme(mode: ThemeMode, content: @Composable () -> Unit) {
    val dark = mode.dark()
    val palette = if (dark) DARK else LIGHT
    MaterialTheme(colorScheme = palette.scheme(dark)) {
        CompositionLocalProvider(LocalContentColor provides palette.text, content = content)
    }
}
