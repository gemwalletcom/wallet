package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.nft.presents.components.NftHeaderActions
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAssetData
import uniffi.gemstone.GemCollectibleAction
import uniffi.gemstone.GemCollectibleAttributeValue
import uniffi.gemstone.GemCollectibleDetails
import uniffi.gemstone.GemCollectibleSection
import uniffi.gemstone.GemListRow
import java.text.DateFormat
import java.util.Date

@Composable
internal fun CollectibleScene(assetData: NFTAssetData, details: GemCollectibleDetails, snackbar: SnackbarHostState, onClose: () -> Unit, onSend: () -> Unit, onAction: (GemCollectibleAction) -> Unit, onOpenAddress: (ChainAddress) -> Unit) {
    Scene(
        titleContent = {
            NftTitle(
                name = assetData.asset.name,
                isVerified = details.isVerified,
                iconSize = compactIconSize,
            )
        },
        onClose = onClose,
        snackbar = snackbar,
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                NftImage(
                    source = assetData.asset.toImageSource(),
                    modifier = Modifier
                        .padding(horizontal = sceneContentPadding())
                        .fillMaxWidth()
                        .aspectRatio(1f)
                        .clip(RoundedCornerShape(paddingDefault)),
                )
            }
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = paddingDefault, bottom = paddingSmall),
                    contentAlignment = Alignment.Center,
                ) {
                    NftHeaderActions(
                        header = details.header,
                        actions = details.actions,
                        onSend = onSend,
                        onAction = onAction,
                    )
                }
            }
            details.sections.forEach { group ->
                val title = group.title.titleRes()
                when (val section = group.section) {
                    is GemCollectibleSection.Status -> verificationStatusItem(section.status.toPrimitives())

                    is GemCollectibleSection.Info -> itemsPositioned(section.rows) { position, row ->
                        GemListRowView(row = row, listPosition = position, onSelectAddress = { onOpenAddress(ChainAddress(assetData.asset.chain, it)) })
                    }

                    is GemCollectibleSection.Attributes -> {
                        title?.let { item { SubheaderItem(it) } }
                        itemsPositioned(section.attributes) { position, attribute ->
                            ListItem(model = ListItemModel(title = attribute.name, subtitle = attribute.value.text()), listPosition = position)
                        }
                    }

                    is GemCollectibleSection.Links -> {
                        title?.let { item { SubheaderItem(it) } }
                        item { GemListRowView(row = GemListRow.Social(section.links), listPosition = ListPosition.Single) }
                    }
                }
            }
        }
    }
}

private fun GemCollectibleAttributeValue.text(): String = when (this) {
    is GemCollectibleAttributeValue.Text -> value
    is GemCollectibleAttributeValue.Date -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date))
}
