package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset

@Composable
fun AssetListHead(asset: Asset, onClick: (() -> Unit)? = null) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        HeaderIcon(asset)
    }
}
