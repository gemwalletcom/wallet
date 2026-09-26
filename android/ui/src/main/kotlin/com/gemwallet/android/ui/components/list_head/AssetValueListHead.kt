package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable
import uniffi.gemstone.GemValueHeader

@Composable
fun AssetValueListHead(header: GemValueHeader) {
    ValueListHead(header = header)
}
