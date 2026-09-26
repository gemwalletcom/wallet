package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.components.image.NftImageSource
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemTransactionHeader

@Composable
fun TransactionListHead(header: GemTransactionHeader, onClick: (() -> Unit)? = null, onAssetClick: ((AssetId) -> Unit)? = null) {
    when (header) {
        is GemTransactionHeader.Amount -> ValueListHead(header = header.header, onClick = onClick)
        is GemTransactionHeader.Value -> ValueListHead(header = header.header, onClick = onClick)
        is GemTransactionHeader.Swap -> SwapListHead(from = header.from, to = header.to, onSwapClick = onClick, onAssetClick = onAssetClick)
        is GemTransactionHeader.Nft -> NftHead(source = NftImageSource(url = header.imageUrl, name = header.name.orEmpty()), onClick = onClick)
        is GemTransactionHeader.AssetImage -> AssetListHead(icon = header.icon, onClick = onClick)
    }
}
