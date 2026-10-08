package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.paddingDefault
import uniffi.gemstone.GemAssetIcon

@Composable
fun AssetListHead(icon: GemAssetIcon, onClick: (() -> Unit)? = null) {
    val interactionSource = remember { MutableInteractionSource() }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .headerClick(interactionSource, onClick)
            .padding(paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box {
            HeaderIcon(icon)
            if (onClick != null) {
                HeaderHighlight(interactionSource, CircleShape)
            }
        }
    }
}
