@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.CircularWavyProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Text
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.features.assets.views.components.AssetsListFooter
import com.gemwallet.android.features.assets.views.components.assets
import com.gemwallet.android.features.banner.views.BannersScene
import com.gemwallet.android.features.banner.views.WelcomeBanner
import com.gemwallet.android.features.perpetual.views.PerpetualsPreviewSection
import com.gemwallet.android.features.update_app.presents.InAppUpdateBanner
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner

fun LazyListScope.walletAssetsItems(
    importing: Boolean,
    showWelcomeBanner: Boolean,
    pinnedAssets: List<AssetInfoDataAggregate>,
    unpinnedAssets: List<AssetInfoDataAggregate>,
    longPressState: MutableState<AssetId?>,
    assetActions: AssetContextActions,
    onAction: (WalletAction) -> Unit,
    onBanner: (Banner) -> Unit,
    onCloseWelcome: () -> Unit,
) {
    val onAssetClick: (AssetId) -> Unit = { onAction(WalletAction.OpenAsset(it)) }
    if (showWelcomeBanner) {
        item {
            WelcomeBanner(
                onBuy = { onAction(WalletAction.Buy) },
                onReceive = { onAction(WalletAction.Receive) },
                onClose = onCloseWelcome,
            )
        }
    }
    item {
        InAppUpdateBanner()
    }
    item {
        BannersScene(asset = null, onClick = onBanner, false)
    }
    if (importing) {
        item {
            Row(
                modifier = Modifier.padding(paddingDefault),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(paddingSmall),
            ) {
                Text(text = "${stringResource(R.string.common_loading)}…")
                CircularWavyProgressIndicator(
                    modifier = Modifier.size(paddingDefault),
                    stroke = Stroke(
                        width = with(LocalDensity.current) { 2.dp.toPx() },
                        cap = StrokeCap.Round,
                    ),
                    trackStroke = Stroke(
                        width = with(LocalDensity.current) { 2.dp.toPx() },
                        cap = StrokeCap.Round,
                    ),
                )
            }
        }
    }
    item {
        PerpetualsPreviewSection(
            onOpenPerpetuals = { onAction(WalletAction.Perpetuals) },
            onOpenPerpetualDetails = { onAction(WalletAction.OpenPerpetualDetails(it)) },
        )
    }
    assets(
        items = pinnedAssets,
        longPressState = longPressState,
        group = AssetsGroupType.Pinned,
        onAssetClick = onAssetClick,
        actions = assetActions,
    )
    assets(
        items = unpinnedAssets,
        longPressState = longPressState,
        group = AssetsGroupType.None,
        onAssetClick = onAssetClick,
        actions = assetActions,
    )
    item { AssetsListFooter { onAction(WalletAction.Manage) } }
}
