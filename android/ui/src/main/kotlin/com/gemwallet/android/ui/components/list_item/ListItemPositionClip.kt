package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingSmall

private val normalTopPadding = 8.dp
private val normalBottomPadding = 8.dp

private val largeCornerRadius = 16.dp
private val smallCornerRadius = 2.dp
private val itemPadding = 1.dp

private val firstItemShape = RoundedCornerShape(topStart = largeCornerRadius, topEnd = largeCornerRadius, bottomStart = smallCornerRadius, bottomEnd = smallCornerRadius)
private val lastItemShape = RoundedCornerShape(bottomStart = largeCornerRadius, bottomEnd = largeCornerRadius, topStart = smallCornerRadius, topEnd = smallCornerRadius)

private val middleItemShape = RoundedCornerShape(smallCornerRadius)
private val singleItemShape = RoundedCornerShape(largeCornerRadius)

private fun ListPosition.topPadding(paddingVertical: Dp?) = when (this) {
    ListPosition.Subhead, ListPosition.First, ListPosition.Single -> paddingVertical ?: normalTopPadding
    ListPosition.Middle, ListPosition.Last -> paddingVertical ?: itemPadding
}

private fun ListPosition.bottomPadding(paddingVertical: Dp?) = when (this) {
    ListPosition.Single -> paddingVertical ?: normalBottomPadding
    ListPosition.Last -> normalBottomPadding
    else -> 0.dp
}

private fun ListPosition.shape() = when (this) {
    ListPosition.Subhead -> null
    ListPosition.First -> firstItemShape
    ListPosition.Middle -> middleItemShape
    ListPosition.Single -> singleItemShape
    ListPosition.Last -> lastItemShape
}

@Composable
fun Modifier.listItem(
    position: ListPosition = ListPosition.Single,
    background: Color = MaterialTheme.colorScheme.background,
    paddingVertical: Dp? = null,
    paddingHorizontal: Dp? = null,
): Modifier {
    val positionedModifier = this
        .padding(top = position.topPadding(paddingVertical), bottom = position.bottomPadding(paddingVertical))
        .let { modifier -> position.shape()?.let(modifier::clip) ?: modifier }

    return padding(horizontal = paddingHorizontal ?: adaptivePadding(default = largeCornerRadius, compact = paddingSmall))
        .then(positionedModifier)
        .then(if (position == ListPosition.Subhead) Modifier else Modifier.background(background))
}
