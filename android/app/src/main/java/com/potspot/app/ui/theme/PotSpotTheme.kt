package com.potspot.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFFf09040),
    onPrimary = Color(0xFF1e1e1e),
    primaryContainer = Color(0xFFcc7722),
    secondary = Color(0xFFf5a623),
    background = Color(0xFF1e1e1e),
    surface = Color(0xFF252526),
    surfaceVariant = Color(0xFF2a2a2b),
    onBackground = Color(0xFFd4d4d4),
    onSurface = Color(0xFFd4d4d4),
    outline = Color(0xFF3e3e42),
    error = Color(0xFFe06c75),
)

@Composable
fun PotSpotTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        content = content,
    )
}
