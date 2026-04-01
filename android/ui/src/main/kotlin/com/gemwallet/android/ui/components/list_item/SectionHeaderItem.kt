package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault

val sectionHeaderHorizontalPadding = paddingDefault * 2

@Composable
fun Modifier.sectionHeaderItem(): Modifier = fillMaxWidth().listItem(
    position = ListPosition.Subhead,
    paddingHorizontal = sectionHeaderHorizontalPadding,
)
