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
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingSmall

import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space8
private val normalTopPadding = space8
private val normalBottomPadding = space8

private val largeCornerRadius = padding16
private val smallCornerRadius = space2
private val itemPadding = space1

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
    else -> space0
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
