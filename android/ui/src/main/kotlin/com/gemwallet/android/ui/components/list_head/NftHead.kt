package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.NftImageSource
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.headerLargeImageSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

@Composable
fun NftHead(
    source: NftImageSource,
    size: Dp = headerLargeImageSize,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = paddingDefault, end = paddingDefault, bottom = paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        NftImage(
            source = source,
            modifier = Modifier
                .size(size)
                .clip(RoundedCornerShape(size / 4)),
        )
        if (source.name.isNotBlank()) {
            Spacer16()
            Text(
                text = source.name,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
                maxLines = 1,
                overflow = TextOverflow.MiddleEllipsis,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
fun NftHead(nftAsset: NFTAsset, size: Dp = headerLargeImageSize) =
    NftHead(nftAsset.toImageSource(), size)

@Composable
fun NftHead(metadata: TransactionNFTTransferMetadata, size: Dp = headerLargeImageSize) =
    NftHead(metadata.toImageSource(), size)
