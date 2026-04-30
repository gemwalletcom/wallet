package com.gemwallet.android.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.unit.dp

val pendingColor = Color(0xffff9314)
val emptyImageColor = Color(0xFF767A81)
const val alpha10 = 0.1f
const val alpha20 = 0.2f
const val alpha50 = 0.5f
const val alpha70 = 0.7f
const val alpha90 = 0.9f

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFF2D5BE6),
    secondary = Color(0xFF808d99), //Color(0xFF818181),
    tertiary = Color(0xFF1B9A6C),
    background = Color(0xFF24262A),
    surface = Color(0xFF1A191A),
    surfaceContainerHighest = Color(0xFF2C2C2E),
    onPrimary = Color.White,
    onSecondary = Color.White,
    onTertiary = Color.White,
    onBackground = Color(0xFF818181),
    error = Color(0xFFF84E4E),
    outline = Color(0xFF767A81),
    outlineVariant = Color(0xFF3A3A3C),
    scrim = Color(0xff34373d), // Header button actions color
)

private val LightColorScheme = lightColorScheme(
    primary = Color(0xFF2D5BE6),
    secondary = Color(0xFF999999), //Color(0xFF818181),
    tertiary = Color(0xFF1B9A6C),
    background = Color(0xFFFFFFFF),
    surface = Color(0xFFF0F0F5),
    surfaceContainerHighest = Color(0xFFE5E5EA),
//    onPrimary = Color.White,
//    onSecondary = Color.White,
//    onTertiary = Color.White,
    onBackground = Color(0xFF1A191A),
    onSurface = Color(0xFF1C1B1F),
    error = Color(0xFFF84E4E),
    outline = Color(0xFF767A81),
    outlineVariant = Color(0xFFD1D1D6),
    scrim = Color(0xffededed),//from #f2f2f2
)

enum class WindowDimension {
    Width,
    Height,
}

@Composable
fun isCompactDimension(dimension: WindowDimension): Boolean {
    val density = LocalDensity.current
    val container = LocalWindowInfo.current.containerSize
    return when (dimension) {
        WindowDimension.Width -> with(density) {
            container.width.toDp()
        } <= SceneSizing.contentMaxWidth

        WindowDimension.Height -> with(density) {
            container.height.toDp()
        } < SceneSizing.compactHeight
    }
}

@Composable
fun WalletTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
