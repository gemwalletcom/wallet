package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import coil3.transform.RoundedCornersTransformation
import com.gemwallet.android.domains.asset.getImageUrl
import com.gemwallet.android.ui.components.DisplayText
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.headerLargeImageSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingLarge
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

@Composable
fun NftHead(
    name: String?,
    imageUrl: String,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = paddingDefault, end = paddingDefault, bottom = paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        AsyncImage(
            size = headerLargeImageSize,
            model = imageUrl,
            placeholderText = name,
            transformation = RoundedCornersTransformation(paddingLarge.value, paddingLarge.value, paddingLarge.value, paddingLarge.value),
            contentDescription = "header_icon",
        )
        if (name != null) {
            Spacer16()
            DisplayText(text = name, modifier = Modifier.fillMaxWidth())
        }
    }
}

@Composable
fun NftHead(nftAsset: NFTAsset) {
    NftHead(name = nftAsset.name, imageUrl = nftAsset.images.preview.url)
}

@Composable
fun NftHead(metadata: TransactionNFTTransferMetadata) {
    NftHead(name = metadata.name, imageUrl = metadata.getImageUrl())
}
