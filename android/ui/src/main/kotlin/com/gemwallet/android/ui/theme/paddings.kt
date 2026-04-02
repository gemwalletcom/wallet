package com.gemwallet.android.ui.theme

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

val extraLargeIconSize = 120.dp
val headerIconSize = 64.dp
val largeIconSize = 96.dp
val headerSmallIconSize = 50.dp
val smallIconSize = 24.dp
val compactIconSize = 20.dp
val tinyIconSize = 16.dp
val listItemIconSize = 44.dp
val headerLargeImageSize = 128.dp
val iconSize = 32.dp

val space0 = 0.dp
val space2 = 2.dp
private val space6 = 6.dp
val space4 = 4.dp
val space8 = 8.dp
val space10 = 10.dp
val space12 = 12.dp
private val space16 = 16.dp
val space24 = 24.dp

val paddingHalfSmall = 4.dp
val paddingSmall = 8.dp
val padding16 = 16.dp
val paddingMiddle = 12.dp
val paddingDefault = 16.dp
val paddingLarge = 32.dp

val mainActionHeight = 48.dp
val sheetCornerSize = 28.dp

object SceneSizing {
    val buttonMaxWidth = 380.dp
    val contentMaxWidth = 360.dp
}

fun Modifier.smallPadding(): Modifier {
    return padding(paddingHalfSmall)
}

fun Modifier.normalPadding(): Modifier {
    return padding(paddingSmall)
}

fun Modifier.middlePadding(): Modifier {
    return padding(paddingMiddle)
}

fun Modifier.defaultPadding(): Modifier {
    return padding(paddingDefault)
}

fun Modifier.largePadding(): Modifier {
    return padding(paddingLarge)
}

@Composable
fun Spacer2() {
    Spacer(modifier = Modifier.size(space2))
}

@Composable
fun Spacer4() {
    Spacer(modifier = Modifier.size(space4))
}

@Composable
fun Spacer6() {
    Spacer(modifier = Modifier.size(space6))
}

@Composable
fun Spacer8() {
    Spacer(modifier = Modifier.size(space8))
}

@Composable
fun Spacer16() {
    Spacer(modifier = Modifier.size(space16))
}

fun LazyListScope.listSpacerBig() {
    item {
        Spacer16()
    }
}
