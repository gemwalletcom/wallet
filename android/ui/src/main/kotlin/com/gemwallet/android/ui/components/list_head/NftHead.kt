package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import com.gemwallet.android.ui.components.DisplayText
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.NftImageSource
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.headerLargeImageSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingLarge
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

@Composable
fun NftHead(source: NftImageSource) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = paddingDefault, end = paddingDefault, bottom = paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        NftImage(
            source = source,
            modifier = Modifier
                .size(headerLargeImageSize)
                .clip(RoundedCornerShape(paddingLarge)),
        )
        if (source.name.isNotBlank()) {
            Spacer16()
            DisplayText(text = source.name, modifier = Modifier.fillMaxWidth())
        }
    }
}

@Composable
fun NftHead(nftAsset: NFTAsset) = NftHead(nftAsset.toImageSource())

@Composable
fun NftHead(metadata: TransactionNFTTransferMetadata) = NftHead(metadata.toImageSource())
