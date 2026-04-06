package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalUriHandler
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.linkType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.features.nft.viewmodels.NftAssetDetailsUIModel
import com.gemwallet.android.features.nft.viewmodels.NftDetailsViewModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.LinkType
import com.wallet.core.primitives.NFTAttribute

@Composable
fun NFTDetailsScene(
    cancelAction: CancelAction,
    onRecipient: (AssetId, String) -> Unit,
) {
    val viewModel: NftDetailsViewModel = hiltViewModel()
    val assetData by viewModel.nftAsset.collectAsStateWithLifecycle()

    val uriHandler = LocalUriHandler.current

    if (assetData == null) {
        return
    }
    val model = assetData!!
    Scene(
        titleContent = {
            NftTitle(
                name = model.assetName,
                status = model.collection.status,
                iconSize = compactIconSize,
            )
        },
        actions = {
            if (assetData?.asset?.chain == Chain.Ethereum) {
                IconButton( { onRecipient(AssetId(model.asset.chain), model.asset.id) } ) {
                    Icon(Icons.Default.ArrowUpward, contentDescription = "Send nft")
                }
            }
        },
        onClose = { cancelAction() },
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
            item { Spacer(Modifier.height(paddingSmall)) }
            generalInfo(model)
            nftAttributes(model.attributes)
            nftLinks(model.collection.links) { uriHandler.openUri(it) }
        }
    }
}

private fun LazyListScope.generalInfo(model: NftAssetDetailsUIModel) {
    item {
        PropertyItem(R.string.nft_collection, model.collection.name, listPosition = ListPosition.First)
        PropertyNetworkItem(model.collection.chain, listPosition = ListPosition.Middle)
        model.asset.contractAddress?.let {
            AddressPropertyItem(
                title = R.string.asset_contract,
                displayText = AddressFormatter(it, chain = model.collection.chain).value(),
                copyValue = it,
                explorerLink = model.contractExplorerLink,
                listPosition = ListPosition.Middle,
            )
        }
        val tokenId = model.asset.tokenId
        val tokenIdDisplayText = if (tokenId.length > 16) {
            AddressFormatter(tokenId, chain = model.collection.chain).value()
        } else {
            "#$tokenId"
        }
        AddressPropertyItem(
            title = R.string.asset_token_id,
            displayText = tokenIdDisplayText,
            copyValue = tokenId,
            explorerLink = model.tokenIdExplorerLink,
            listPosition = ListPosition.Last,
        )
    }
}

private fun LazyListScope.nftAttributes(attributes: List<NFTAttribute>) {
    item {
        SubheaderItem(R.string.nft_properties)
    }
    itemsPositioned(attributes) { position, item ->
        PropertyItem(item.name, item.value, listPosition = position)
    }
}

private fun LazyListScope.nftLinks(links: List<AssetLink>, onLinkClick: (String) -> Unit) {
    if (links.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.social_links)
    }

    val links = links.sortedWith { l, r ->
        if (r.linkType == LinkType.Website) {
            0
        } else {
            r.name.compareTo(l.name)
        }
    }.map {
        when (it.linkType) {
            LinkType.Coingecko -> Triple(it.url, R.string.social_coingecko, R.drawable.coingecko)
            LinkType.X -> Triple(it.url, R.string.social_x, R.drawable.twitter)
            LinkType.Telegram -> Triple(it.url, R.string.social_telegram, R.drawable.telegram)
            LinkType.GitHub -> Triple(it.url, R.string.social_github, R.drawable.github)
            LinkType.Instagram -> Triple(it.url, R.string.social_instagram, R.drawable.instagram)
            LinkType.OpenSea -> Triple(it.url, R.string.social_opensea, R.drawable.opensea)
            LinkType.MagicEden -> Triple(it.url, R.string.social_magiceden, R.drawable.magiceden)
            LinkType.CoinMarketCap -> Triple(it.url, R.string.social_coinmarketcap, R.drawable.coinmarketcap)
            LinkType.TikTok -> Triple(it.url, R.string.social_tiktok, R.drawable.tiktok)
            LinkType.Discord -> Triple(it.url, R.string.social_discord, R.drawable.discord)
            else -> Triple(it.url, R.string.social_website, R.drawable.website)
        }
    }

    itemsPositioned(links) { position, item ->
        val (url, title, icon) = item
        PropertyItem(title, icon, listPosition = position) { onLinkClick(url) }
    }
}
