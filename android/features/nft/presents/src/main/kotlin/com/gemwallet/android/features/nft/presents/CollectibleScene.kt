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
import com.gemwallet.android.features.nft.presents.components.NftHeaderActions
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.features.nft.viewmodels.models.CollectibleUIModel
import com.gemwallet.android.features.nft.viewmodels.models.NftSectionUIModel
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemCollectibleAction

@Composable
internal fun CollectibleScene(model: CollectibleUIModel, snackbar: SnackbarHostState, onClose: () -> Unit, onSend: () -> Unit, onAction: (GemCollectibleAction) -> Unit, onOpenAddress: (ChainAddress) -> Unit) {
    Scene(
        titleContent = {
            NftTitle(
                name = model.asset.name,
                isVerified = model.isVerified,
                iconSize = compactIconSize,
            )
        },
        onClose = onClose,
        snackbar = snackbar,
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                NftImage(
                    source = model.asset.toImageSource(),
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
                        header = model.header,
                        actions = model.actions,
                        onSend = onSend,
                        onAction = onAction,
                    )
                }
            }
            model.sections.forEach { section ->
                when (section) {
                    is NftSectionUIModel.Status -> verificationStatusItem(section.status)

                    is NftSectionUIModel.Info -> itemsPositioned(section.rows) { position, row ->
                        GemListRowView(row = row, listPosition = position, onSelectAddress = { onOpenAddress(ChainAddress(model.asset.chain, it)) })
                    }

                    is NftSectionUIModel.Attributes -> {
                        item { SubheaderItem(section.title) }
                        itemsPositioned(section.rows) { position, row -> ListItem(model = row, listPosition = position) }
                    }

                    is NftSectionUIModel.Links -> {
                        item { SubheaderItem(section.title) }
                        item { GemListRowView(row = section.row, listPosition = ListPosition.Single) }
                    }
                }
            }
        }
    }
}
