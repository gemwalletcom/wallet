package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

val sectionHeaderHorizontalPadding: Dp
    @Composable get() = adaptivePadding(default = paddingDefault, compact = paddingSmall) + paddingMiddle

@Composable
fun Modifier.sectionHeaderItem(paddingVertical: Dp? = null): Modifier = fillMaxWidth().listItem(
    position = ListPosition.Subhead,
    paddingVertical = paddingVertical,
    paddingHorizontal = sectionHeaderHorizontalPadding,
)
